//! Writes a run's usage to the journal (`storage::usage`) in the background, so runs never wait
//! for the database. Events arrive from the run's accounting (`BudgetTracker`).

use std::sync::Arc;

use tokio::sync::{mpsc, oneshot};
use tracing::error;

use crate::storage::repository::Repository;
use crate::storage::usage::{UsageEvent, UsageRun};

enum Message {
    Event(Box<UsageEvent>),
    Flush(oneshot::Sender<()>),
}

/// The journal of one run; clones share the writer.
#[derive(Debug, Clone)]
pub struct UsageJournal {
    sender: mpsc::UnboundedSender<Message>,
    run_id: String,
    run_type: String,
}

impl std::fmt::Debug for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Message::Event(_) => "Event",
            Message::Flush(_) => "Flush",
        })
    }
}

impl UsageJournal {
    /// Names the run for the statistics and starts its writer.
    pub fn start(repo: Arc<Repository>, run: UsageRun) -> Self {
        let (sender, mut receiver) = mpsc::unbounded_channel::<Message>();
        let journal = Self { sender, run_id: run.run_id.clone(), run_type: run.run_type.clone() };
        // Without a runtime (synchronous unit tests) nothing is written.
        let Ok(runtime) = tokio::runtime::Handle::try_current() else {
            return journal;
        };
        runtime.spawn(async move {
            if let Err(e) = repo.remember_usage_run(&run).await {
                error!(error = %e, "Failed to name the run in the usage journal");
            }
            while let Some(first) = receiver.recv().await {
                // Everything already waiting goes in one transaction.
                let mut batch = Vec::new();
                let mut flushed = Vec::new();
                for message in std::iter::once(first).chain(std::iter::from_fn(|| receiver.try_recv().ok())) {
                    match message {
                        Message::Event(event) => batch.push(*event),
                        Message::Flush(done) => flushed.push(done),
                    }
                }
                if !batch.is_empty() {
                    if let Err(e) = repo.record_usage(&batch).await {
                        error!(error = %e, "Failed to write the usage journal");
                    }
                }
                for done in flushed {
                    let _ = done.send(());
                }
            }
        });
        journal
    }

    /// The journal of a run starting now.
    pub fn for_run(repo: Arc<Repository>, run_id: &str, run_type: &str, title: &str) -> Self {
        Self::start(
            repo,
            UsageRun { run_id: run_id.into(), run_type: run_type.into(), title: title.into(), started_at: chrono::Utc::now().timestamp() },
        )
    }

    /// Records an event of this run, dated now.
    pub fn record(&self, mut event: UsageEvent) {
        event.at = chrono::Utc::now().timestamp();
        event.run_id = Some(self.run_id.clone());
        event.run_type = self.run_type.clone();
        let _ = self.sender.send(Message::Event(Box::new(event)));
    }

    /// Waits until everything recorded so far is written.
    pub async fn flush(&self) {
        let (done, written) = oneshot::channel();
        if self.sender.send(Message::Flush(done)).is_ok() {
            let _ = written.await;
        }
    }
}
