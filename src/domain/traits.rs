use async_trait::async_trait;
use crate::domain::errors::DomainError;
use crate::domain::models::{LogEntry, PipelineContext};

#[async_trait]
pub trait Parser<T>: Send + Sync {
    async fn parse(&self, raw: &str) -> Result<T, DomainError>;
}

#[async_trait]
pub trait Middleware: Send + Sync {
    async fn process(
        &mut self,
        ctx: &mut PipelineContext,
    ) -> Result<(), DomainError>;

    fn name(&self) -> &str;
}

#[async_trait]
pub trait Sink: Send + Sync {
    async fn write(&mut self, entry: &LogEntry) -> Result<(), DomainError>;
    async fn flush(&mut self) -> Result<(), DomainError>;
}
