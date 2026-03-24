use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};

use crate::domain::{DomainError, LogEntry, Parser};

pub enum LogFormat {
    Json,
    PlainText,
}

pub struct FileReader {
    path: PathBuf,
    parser: Box<dyn Parser<LogEntry>>,
}

impl FileReader {
    pub fn new(path: PathBuf, parser: Box<dyn Parser<LogEntry>>) -> Self {
        Self { path, parser }
    }

    pub async fn read_all(&self) -> Result<Vec<LogEntry>, DomainError> {
        let file = File::open(&self.path).await.map_err(|e| DomainError::IoError {
            source: e,
        })?;

        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        let mut entries = Vec::new();
        let mut line_number: u32 = 0;

        while let Some(line) = lines.next_line().await.map_err(|e| DomainError::IoError {
            source: e,
        })? {
            line_number += 1;
            let trimmed = line.trim().to_string();

            if trimmed.is_empty() {
                continue;
            }

            match self.parser.parse(&trimmed).await {
                Ok(entry) => entries.push(entry),
                Err(DomainError::ParseError { format, message }) => {
                    eprintln!(
                        "[WARN] Skipping line {}: parse error in '{}': {}",
                        line_number, format, message
                    );
                }
                Err(e) => return Err(e),
            }
        }

        Ok(entries)
    }
}
