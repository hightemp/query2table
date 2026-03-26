use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, warn};

use crate::providers::llm::manager::LlmManager;
use crate::storage::models::SchemaColumn;
use crate::roles::document_parser::ParsedDocument;
use crate::roles::extractor::{Extractor, ExtractedRow};

/// A document to extract entities from.
#[derive(Debug, Clone)]
pub struct ExtractionJob {
    pub fetched_page_id: String,
    pub document: ParsedDocument,
}

/// Result of an extraction operation.
#[derive(Debug)]
pub struct ExtractionOutput {
    pub fetched_page_id: String,
    pub page_url: String,
    pub rows: Vec<ExtractedRow>,
}

/// Error result for a failed extraction.
#[derive(Debug)]
pub struct ExtractionFailure {
    pub fetched_page_id: String,
    pub page_url: String,
    pub error: String,
}

/// Result from the extract pool.
#[derive(Debug)]
pub enum ExtractResult {
    Success(ExtractionOutput),
    Failure(ExtractionFailure),
}

/// Spawns a pool of extraction workers that pull documents from a channel,
/// run LLM extraction, and send results to an output channel.
pub fn spawn_extract_pool(
    llm: Arc<LlmManager>,
    columns: Vec<SchemaColumn>,
    num_workers: usize,
) -> (mpsc::Sender<ExtractionJob>, mpsc::Receiver<ExtractResult>) {
    let (job_tx, job_rx) = mpsc::channel::<ExtractionJob>(num_workers * 4);
    let (result_tx, result_rx) = mpsc::channel::<ExtractResult>(num_workers * 4);

    let job_rx = Arc::new(tokio::sync::Mutex::new(job_rx));
    let columns = Arc::new(columns);

    for worker_id in 0..num_workers {
        let llm = llm.clone();
        let job_rx = job_rx.clone();
        let result_tx = result_tx.clone();
        let columns = columns.clone();

        tokio::spawn(async move {
            debug!(worker_id, "Extract worker started");
            loop {
                let job = {
                    let mut rx = job_rx.lock().await;
                    rx.recv().await
                };
                let job = match job {
                    Some(j) => j,
                    None => {
                        debug!(worker_id, "Extract worker shutting down (channel closed)");
                        break;
                    }
                };

                let page_url = job.document.url.clone();
                let result = Extractor::extract(&job.document, &columns, &llm).await;

                let extract_result = match result {
                    Ok(extraction) => {
                        debug!(
                            worker_id,
                            url = %page_url,
                            rows = extraction.rows.len(),
                            "Extracted entities"
                        );
                        ExtractResult::Success(ExtractionOutput {
                            fetched_page_id: job.fetched_page_id,
                            page_url: extraction.page_url,
                            rows: extraction.rows,
                        })
                    }
                    Err(e) => {
                        warn!(
                            worker_id,
                            url = %page_url,
                            error = %e,
                            "Extraction failed"
                        );
                        ExtractResult::Failure(ExtractionFailure {
                            fetched_page_id: job.fetched_page_id,
                            page_url,
                            error: e.to_string(),
                        })
                    }
                };

                if result_tx.send(extract_result).await.is_err() {
                    debug!(worker_id, "Result channel closed, stopping worker");
                    break;
                }
            }
        });
    }

    (job_tx, result_rx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extraction_job_clone() {
        let job = ExtractionJob {
            fetched_page_id: "fp-1".into(),
            document: ParsedDocument {
                title: "Test".into(),
                text: "Hello".into(),
                url: "https://example.com".into(),
            },
        };
        let cloned = job.clone();
        assert_eq!(cloned.fetched_page_id, "fp-1");
        assert_eq!(cloned.document.url, "https://example.com");
    }

    #[test]
    fn test_extract_result_variants() {
        let success = ExtractResult::Success(ExtractionOutput {
            fetched_page_id: "fp-1".into(),
            page_url: "https://example.com".into(),
            rows: vec![],
        });
        assert!(matches!(success, ExtractResult::Success(_)));

        let failure = ExtractResult::Failure(ExtractionFailure {
            fetched_page_id: "fp-2".into(),
            page_url: "https://bad.com".into(),
            error: "LLM error".into(),
        });
        assert!(matches!(failure, ExtractResult::Failure(_)));
    }

    #[tokio::test]
    async fn test_extraction_job_channel() {
        let (tx, mut rx) = mpsc::channel::<ExtractionJob>(16);
        let job = ExtractionJob {
            fetched_page_id: "fp-1".into(),
            document: ParsedDocument {
                title: "Test".into(),
                text: "Content".into(),
                url: "https://example.com".into(),
            },
        };
        tx.send(job).await.unwrap();
        let received = rx.recv().await.unwrap();
        assert_eq!(received.fetched_page_id, "fp-1");
    }
}
