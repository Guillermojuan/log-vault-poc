#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("Parse error in format '{format}': {message}")]
    ParseError {
        format: String,
        message: String,
    },
    #[error("Invalid log level: '{value}'")]
    InvalidLogLevel {
        value: String,
    },
    #[error("Pipeline stage '{stage}' failed: {message}")]
    StageError {
        stage: String,
        message: String,
    },
    #[error("Sink write failed")]
    SinkError {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    #[error("I/O error")]
    IoError {
        #[source]
        source: std::io::Error,
    },
}
