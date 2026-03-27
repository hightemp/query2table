use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc;
use tracing::{debug, warn};

use crate::providers::http::client::{HttpFetcher, FetchError};
use crate::roles::document_parser::{DocumentParser, ParsedDocument};

/// A URL to fetch, with metadata for tracking.
#[derive(Debug, Clone)]
pub struct FetchJob {
    pub search_result_id: String,
    pub url: String,
    pub title: String,
}

/// Result of a fetch+parse operation.
#[derive(Debug)]
pub struct FetchedDocument {
    pub search_result_id: String,
    pub document: ParsedDocument,
    pub fetch_duration_ms: u64,
    pub http_status: u16,
    pub content_length: usize,
}

/// Error result for a failed fetch.
#[derive(Debug)]
pub struct FetchFailure {
    pub search_result_id: String,
    pub url: String,
    pub error: String,
}

/// Result from the fetch pool: either success or failure.
#[derive(Debug)]
pub enum FetchResult {
    Success(FetchedDocument),
    Failure(FetchFailure),
}

/// Spawns a pool of fetch workers that pull URLs from a channel,
/// fetch+parse them, and send results to an output channel.
///
/// Returns the sender for submitting jobs and receiver for collecting results.
pub fn spawn_fetch_pool(
    fetcher: Arc<HttpFetcher>,
    num_workers: usize,
) -> (mpsc::Sender<FetchJob>, mpsc::Receiver<FetchResult>) {
    let (job_tx, job_rx) = mpsc::channel::<FetchJob>(num_workers * 4);
    let (result_tx, result_rx) = mpsc::channel::<FetchResult>(num_workers * 4);

    // Wrap receiver in Arc<Mutex> so multiple workers can pull from it
    let job_rx = Arc::new(tokio::sync::Mutex::new(job_rx));

    for worker_id in 0..num_workers {
        let fetcher = fetcher.clone();
        let job_rx = job_rx.clone();
        let result_tx = result_tx.clone();

        tokio::spawn(async move {
            debug!(worker_id, "Fetch worker started");
            loop {
                let job = {
                    let mut rx = job_rx.lock().await;
                    rx.recv().await
                };
                let job = match job {
                    Some(j) => j,
                    None => {
                        debug!(worker_id, "Fetch worker shutting down (channel closed)");
                        break;
                    }
                };

                let start = Instant::now();
                let result = fetch_and_parse(&fetcher, &job).await;
                let duration_ms = start.elapsed().as_millis() as u64;
                if duration_ms > 30_000 {
                    warn!(worker_id, url = %job.url, duration_ms, "Slow fetch detected");
                }

                let fetch_result = match result {
                    Ok((page_body, http_status)) => {
                        let doc = DocumentParser::parse(&page_body, &job.url);
                        debug!(
                            worker_id,
                            url = %job.url,
                            text_len = doc.text.len(),
                            duration_ms,
                            "Fetched and parsed"
                        );
                        FetchResult::Success(FetchedDocument {
                            search_result_id: job.search_result_id,
                            document: doc,
                            fetch_duration_ms: duration_ms,
                            http_status,
                            content_length: page_body.len(),
                        })
                    }
                    Err(e) => {
                        warn!(
                            worker_id,
                            url = %job.url,
                            error = %e,
                            "Fetch failed"
                        );
                        FetchResult::Failure(FetchFailure {
                            search_result_id: job.search_result_id,
                            url: job.url,
                            error: e.to_string(),
                        })
                    }
                };

                if result_tx.send(fetch_result).await.is_err() {
                    debug!(worker_id, "Result channel closed, stopping worker");
                    break;
                }
            }
        });
    }

    (job_tx, result_rx)
}

async fn fetch_and_parse(
    fetcher: &HttpFetcher,
    job: &FetchJob,
) -> Result<(String, u16), FetchError> {
    let page = fetcher.fetch(&job.url).await?;
    Ok((page.body, page.status))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fetch_job_clone() {
        let job = FetchJob {
            search_result_id: "sr-1".into(),
            url: "https://example.com".into(),
            title: "Example".into(),
        };
        let cloned = job.clone();
        assert_eq!(cloned.url, "https://example.com");
    }

    #[test]
    fn test_fetch_result_variants() {
        let success = FetchResult::Success(FetchedDocument {
            search_result_id: "sr-1".into(),
            document: ParsedDocument {
                title: "Test".into(),
                text: "Hello world".into(),
                url: "https://example.com".into(),
            },
            fetch_duration_ms: 200,
            http_status: 200,
            content_length: 1000,
        });
        assert!(matches!(success, FetchResult::Success(_)));

        let failure = FetchResult::Failure(FetchFailure {
            search_result_id: "sr-2".into(),
            url: "https://bad.com".into(),
            error: "timeout".into(),
        });
        assert!(matches!(failure, FetchResult::Failure(_)));
    }

    #[tokio::test]
    async fn test_channel_capacity() {
        // Verify channel creation works with expected capacity
        let (tx, mut rx) = mpsc::channel::<FetchJob>(32);
        let job = FetchJob {
            search_result_id: "sr-1".into(),
            url: "https://example.com".into(),
            title: "Test".into(),
        };
        tx.send(job).await.unwrap();
        let received = rx.recv().await.unwrap();
        assert_eq!(received.url, "https://example.com");
    }
}
