use crate::domain::{DomainError, LogEntry, Middleware, PipelineContext, Sink};

pub struct Pipeline {
    stages: Vec<Box<dyn Middleware>>,
    sink: Box<dyn Sink>,
}

impl Pipeline {
    pub fn new(stages: Vec<Box<dyn Middleware>>, sink: Box<dyn Sink>) -> Self {
        Self { stages, sink }
    }

    pub async fn run(&mut self, entries: Vec<LogEntry>) -> Result<PipelineStats, DomainError> {
        let mut stats = PipelineStats::default();

        for entry in entries {
            stats.total += 1;
            let mut ctx = PipelineContext::new(entry);

            for stage in self.stages.iter_mut() {
                if ctx.is_filtered {
                    break;
                }
                stage.process(&mut ctx).await.map_err(|e| DomainError::StageError {
                    stage: stage.name().to_string(),
                    message: e.to_string(),
                })?;
            }

            if ctx.is_filtered {
                stats.filtered += 1;
            } else {
                self.sink.write(&ctx.entry).await?;
                stats.written += 1;
            }
        }

        self.sink.flush().await?;
        Ok(stats)
    }
}

#[derive(Debug, Default)]
pub struct PipelineStats {
    pub total: usize,
    pub filtered: usize,
    pub written: usize,
}

impl std::fmt::Display for PipelineStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Processed: {} total | {} written | {} filtered",
            self.total, self.written, self.filtered
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use chrono::Utc;
    use crate::domain::{DomainError, LogEntry, LogLevel, Sink};
    use crate::application::stages::LevelFilterStage;

    struct SpySink {
        pub received: Vec<LogEntry>,
    }

    impl SpySink {
        fn new() -> Self {
            Self { received: Vec::new() }
        }
    }

    #[async_trait]
    impl Sink for SpySink {
        async fn write(&mut self, entry: &LogEntry) -> Result<(), DomainError> {
            self.received.push(entry.clone());
            Ok(())
        }

        async fn flush(&mut self) -> Result<(), DomainError> {
            Ok(())
        }
    }

    fn make_entry(level: LogLevel, message: &str) -> LogEntry {
        LogEntry::new(Utc::now(), level, message.to_string())
    }

    #[tokio::test]
    async fn test_pipeline_filters_and_writes() {
        let stages: Vec<Box<dyn Middleware>> = vec![
            Box::new(LevelFilterStage::new(LogLevel::Warn)),
        ];
        let sink = Box::new(SpySink::new());
        let mut pipeline = Pipeline::new(stages, sink);

        let entries = vec![
            make_entry(LogLevel::Debug, "mensaje debug"),
            make_entry(LogLevel::Warn,  "advertencia importante"),
            make_entry(LogLevel::Error, "error critico en 2026-03-24"),
        ];

        let stats = pipeline.run(entries).await.unwrap();

        assert_eq!(stats.total,    3);
        assert_eq!(stats.written,  2);
        assert_eq!(stats.filtered, 1);
    }
}
