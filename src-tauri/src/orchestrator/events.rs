use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tracing::debug;

/// Publishes pipeline events to the frontend via Tauri event system.
#[derive(Clone)]
pub struct EventPublisher {
    app: AppHandle,
    run_id: String,
}

impl EventPublisher {
    pub fn new(app: AppHandle, run_id: String) -> Self {
        Self { app, run_id }
    }

    pub fn emit_status_changed(&self, status: &str) {
        let payload = StatusChangedEvent {
            run_id: self.run_id.clone(),
            status: status.to_string(),
        };
        if let Err(e) = self.app.emit("run:status_changed", &payload) {
            tracing::error!(error = %e, "Failed to emit status_changed event");
        }
        debug!(run_id = %self.run_id, status, "Emitted status_changed");
    }

    pub fn emit_row_added(&self, row_id: &str, data: &serde_json::Value, confidence: f64) {
        let payload = RowAddedEvent {
            run_id: self.run_id.clone(),
            row_id: row_id.to_string(),
            data: data.clone(),
            confidence,
        };
        if let Err(e) = self.app.emit("run:row_added", &payload) {
            tracing::error!(error = %e, "Failed to emit row_added event");
        }
        debug!(run_id = %self.run_id, row_id, "Emitted row_added");
    }

    pub fn emit_progress(&self, stats: ProgressStats) {
        let payload = ProgressEvent {
            run_id: self.run_id.clone(),
            stats,
        };
        if let Err(e) = self.app.emit("run:progress_update", &payload) {
            tracing::error!(error = %e, "Failed to emit progress_update event");
        }
    }

    pub fn emit_log(&self, level: &str, role: &str, message: &str) {
        let payload = LogEntryEvent {
            run_id: self.run_id.clone(),
            level: level.to_string(),
            role: role.to_string(),
            message: message.to_string(),
        };
        if let Err(e) = self.app.emit("run:log_entry", &payload) {
            tracing::error!(error = %e, "Failed to emit log_entry event");
        }
    }

    pub fn emit_schema_proposed(&self, columns: &serde_json::Value) {
        let payload = SchemaProposedEvent {
            run_id: self.run_id.clone(),
            columns: columns.clone(),
        };
        if let Err(e) = self.app.emit("run:schema_proposed", &payload) {
            tracing::error!(error = %e, "Failed to emit schema_proposed event");
        }
        debug!(run_id = %self.run_id, "Emitted schema_proposed");
    }

    pub fn emit_error(&self, error: &str) {
        let payload = RunErrorEvent {
            run_id: self.run_id.clone(),
            error: error.to_string(),
        };
        if let Err(e) = self.app.emit("run:error", &payload) {
            tracing::error!(error = %e, "Failed to emit error event");
        }
    }
}

// --- Event payload types ---

#[derive(Debug, Clone, Serialize)]
pub struct StatusChangedEvent {
    pub run_id: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RowAddedEvent {
    pub run_id: String,
    pub row_id: String,
    pub data: serde_json::Value,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProgressEvent {
    pub run_id: String,
    pub stats: ProgressStats,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProgressStats {
    pub rows_found: u64,
    pub pages_fetched: u64,
    pub pages_total: u64,
    pub queries_executed: u64,
    pub queries_total: u64,
    pub elapsed_secs: u64,
    pub spent_usd: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct LogEntryEvent {
    pub run_id: String,
    pub level: String,
    pub role: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SchemaProposedEvent {
    pub run_id: String,
    pub columns: serde_json::Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct RunErrorEvent {
    pub run_id: String,
    pub error: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_stats_serialize() {
        let stats = ProgressStats {
            rows_found: 10,
            pages_fetched: 25,
            pages_total: 50,
            queries_executed: 5,
            queries_total: 8,
            elapsed_secs: 120,
            spent_usd: 0.05,
        };
        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("rows_found"));
        assert!(json.contains("pages_fetched"));
        assert!(json.contains("spent_usd"));
    }

    #[test]
    fn test_status_changed_event_serialize() {
        let event = StatusChangedEvent {
            run_id: "run-1".to_string(),
            status: "running".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("run-1"));
        assert!(json.contains("running"));
    }

    #[test]
    fn test_row_added_event_serialize() {
        let event = RowAddedEvent {
            run_id: "run-1".to_string(),
            row_id: "row-1".to_string(),
            data: serde_json::json!({"name": "Acme Corp"}),
            confidence: 0.95,
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("Acme Corp"));
        assert!(json.contains("0.95"));
    }

    #[test]
    fn test_log_entry_event_serialize() {
        let event = LogEntryEvent {
            run_id: "run-1".to_string(),
            level: "INFO".to_string(),
            role: "interpreter".to_string(),
            message: "Parsed query".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("interpreter"));
        assert!(json.contains("Parsed query"));
    }
}
