use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::paths::{app_support_dir, log_path};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionLogEntry {
    pub timestamp: u64,
    pub action_id: String,
    pub selected_count: usize,
    pub status: ActionLogStatus,
    pub message: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ActionLogStatus {
    Success,
    Failure,
}

#[derive(Debug, thiserror::Error)]
pub enum LoggingError {
    #[error("failed to create app support directory: {0}")]
    CreateDir(#[source] std::io::Error),
    #[error("failed to open actions.jsonl: {0}")]
    Open(#[source] std::io::Error),
    #[error("failed to write actions.jsonl: {0}")]
    Write(#[source] std::io::Error),
    #[error("failed to read actions.jsonl: {0}")]
    Read(#[source] std::io::Error),
    #[error("failed to serialize action log entry: {0}")]
    Serialize(#[source] serde_json::Error),
}

pub type LoggingResult<T> = Result<T, LoggingError>;

impl ActionLogEntry {
    pub fn success(
        action_id: impl Into<String>,
        selected_count: usize,
        message: impl Into<String>,
    ) -> Self {
        Self {
            timestamp: unix_timestamp(),
            action_id: action_id.into(),
            selected_count,
            status: ActionLogStatus::Success,
            message: message.into(),
            error: None,
        }
    }

    pub fn failure(
        action_id: impl Into<String>,
        selected_count: usize,
        message: impl Into<String>,
        error: impl Into<String>,
    ) -> Self {
        Self {
            timestamp: unix_timestamp(),
            action_id: action_id.into(),
            selected_count,
            status: ActionLogStatus::Failure,
            message: message.into(),
            error: Some(error.into()),
        }
    }
}

pub fn append_action_log(entry: &ActionLogEntry) -> LoggingResult<()> {
    fs::create_dir_all(app_support_dir()).map_err(LoggingError::CreateDir)?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path())
        .map_err(LoggingError::Open)?;
    let json = serde_json::to_string(entry).map_err(LoggingError::Serialize)?;
    writeln!(file, "{json}").map_err(LoggingError::Write)
}

pub fn read_recent_logs(limit: usize) -> LoggingResult<Vec<ActionLogEntry>> {
    let path = log_path();
    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = OpenOptions::new()
        .read(true)
        .open(path)
        .map_err(LoggingError::Open)?;
    let reader = BufReader::new(file);
    let lines = reader
        .lines()
        .collect::<Result<Vec<_>, _>>()
        .map_err(LoggingError::Read)?;

    let entries = lines
        .into_iter()
        .rev()
        .take(limit)
        .filter_map(|line| serde_json::from_str::<ActionLogEntry>(&line).ok())
        .collect::<Vec<_>>();

    Ok(entries)
}

fn unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
