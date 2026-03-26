use serde::{Deserialize, Serialize};

/// Run status enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    Pending,
    SchemaReview,
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

impl RunStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::SchemaReview => "schema_review",
            Self::Running => "running",
            Self::Paused => "paused",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(Self::Pending),
            "schema_review" => Some(Self::SchemaReview),
            "running" => Some(Self::Running),
            "paused" => Some(Self::Paused),
            "completed" => Some(Self::Completed),
            "failed" => Some(Self::Failed),
            "cancelled" => Some(Self::Cancelled),
            _ => None,
        }
    }
}

/// Schema column definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(PartialEq)]
pub struct SchemaColumn {
    pub name: String,
    #[serde(rename = "type")]
    pub col_type: String,
    pub description: String,
    pub required: bool,
}

/// Entity row status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EntityRowStatus {
    Raw,
    Validated,
    Deduplicated,
    Final,
}

impl EntityRowStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Raw => "raw",
            Self::Validated => "validated",
            Self::Deduplicated => "deduplicated",
            Self::Final => "final",
        }
    }
}

/// Log level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Debug => "DEBUG",
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_status_roundtrip() {
        let statuses = vec![
            RunStatus::Pending,
            RunStatus::SchemaReview,
            RunStatus::Running,
            RunStatus::Paused,
            RunStatus::Completed,
            RunStatus::Failed,
            RunStatus::Cancelled,
        ];
        for status in statuses {
            let s = status.as_str();
            let parsed = RunStatus::from_str(s).unwrap();
            assert_eq!(parsed, status);
        }
    }

    #[test]
    fn test_run_status_invalid() {
        assert!(RunStatus::from_str("invalid").is_none());
    }

    #[test]
    fn test_schema_column_serialization() {
        let col = SchemaColumn {
            name: "company_name".to_string(),
            col_type: "text".to_string(),
            description: "Name of the company".to_string(),
            required: true,
        };
        let json = serde_json::to_string(&col).unwrap();
        assert!(json.contains("company_name"));
        assert!(json.contains("\"type\":\"text\""));

        let deserialized: SchemaColumn = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "company_name");
        assert_eq!(deserialized.col_type, "text");
        assert!(deserialized.required);
    }
}
