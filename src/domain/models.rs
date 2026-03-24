use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info  => "INFO",
            LogLevel::Warn  => "WARN",
            LogLevel::Error => "ERROR",
            LogLevel::Fatal => "FATAL",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub message: String,
    pub source: Option<String>,
    pub fields: HashMap<String, serde_json::Value>,
}

impl LogEntry {
    pub fn new(timestamp: DateTime<Utc>, level: LogLevel, message: String) -> Self {
        Self {
            timestamp,
            level,
            message,
            source: None,
            fields: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PipelineContext {
    pub entry: LogEntry,
    pub metadata: HashMap<String, String>,
    pub is_filtered: bool,
}

impl PipelineContext {
    pub fn new(entry: LogEntry) -> Self {
        Self {
            entry,
            metadata: HashMap::new(),
            is_filtered: false,
        }
    }

    pub fn filter(&mut self) {
        self.is_filtered = true;
    }
}
