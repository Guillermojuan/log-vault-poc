use async_trait::async_trait;
use tokio::fs::File;
use tokio::io::{AsyncWriteExt, BufWriter};

use crate::domain::{DomainError, LogEntry, Sink};

pub struct JsonFileWriter {
    writer: BufWriter<File>,
}

impl JsonFileWriter {
    pub async fn new(path: &std::path::Path) -> Result<Self, DomainError> {
        let file = File::create(path).await.map_err(|e| DomainError::IoError {
            source: e,
        })?;
        Ok(Self {
            writer: BufWriter::new(file),
        })
    }
}

#[async_trait]
impl Sink for JsonFileWriter {
    async fn write(&mut self, entry: &LogEntry) -> Result<(), DomainError> {
        let mut line = serde_json::to_string(entry).map_err(|e| DomainError::SinkError {
            source: Box::new(e),
        })?;
        line.push('\n');
        self.writer
            .write_all(line.as_bytes())
            .await
            .map_err(|e| DomainError::IoError { source: e })
    }

    async fn flush(&mut self) -> Result<(), DomainError> {
        self.writer
            .flush()
            .await
            .map_err(|e| DomainError::IoError { source: e })
    }
}
