use std::collections::HashMap;
use async_trait::async_trait;

use crate::domain::{DomainError, LogLevel, Middleware, PipelineContext};

pub struct LevelFilterStage {
    minimum_level: LogLevel,
}

impl LevelFilterStage {
    pub fn new(minimum_level: LogLevel) -> Self {
        Self { minimum_level }
    }
}

#[async_trait]
impl Middleware for LevelFilterStage {
    async fn process(&mut self, ctx: &mut PipelineContext) -> Result<(), DomainError> {
        if ctx.entry.level < self.minimum_level {
            ctx.filter();
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "LevelFilterStage"
    }
}

pub struct FieldEnricherStage {
    fields: HashMap<String, serde_json::Value>,
}

impl FieldEnricherStage {
    pub fn new(fields: HashMap<String, serde_json::Value>) -> Self {
        Self { fields }
    }
}

#[async_trait]
impl Middleware for FieldEnricherStage {
    async fn process(&mut self, ctx: &mut PipelineContext) -> Result<(), DomainError> {
        if ctx.is_filtered {
            return Ok(());
        }
        for (key, value) in &self.fields {
            ctx.entry.fields.insert(key.clone(), value.clone());
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "FieldEnricherStage"
    }
}

pub struct KeywordFilterStage {
    keywords: Vec<String>,
    exclude: bool,
}

impl KeywordFilterStage {
    pub fn new(keywords: Vec<String>, exclude: bool) -> Self {
        Self { keywords, exclude }
    }
}

#[async_trait]
impl Middleware for KeywordFilterStage {
    async fn process(&mut self, ctx: &mut PipelineContext) -> Result<(), DomainError> {
        if ctx.is_filtered {
            return Ok(());
        }

        let message_lower = ctx.entry.message.to_lowercase();
        let matches = self
            .keywords
            .iter()
            .any(|kw| message_lower.contains(&kw.to_lowercase()));

        if self.exclude && matches {
            ctx.filter();
        } else if !self.exclude && !matches {
            ctx.filter();
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "KeywordFilterStage"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use crate::domain::{LogEntry, LogLevel, PipelineContext};

    fn make_context(level: LogLevel, message: &str) -> PipelineContext {
        let entry = LogEntry::new(Utc::now(), level, message.to_string());
        PipelineContext::new(entry)
    }

    #[tokio::test]
    async fn test_level_filter_removes_below_minimum() {
        let mut stage = LevelFilterStage::new(LogLevel::Warn);
        let mut ctx = make_context(LogLevel::Debug, "mensaje de debug");
        stage.process(&mut ctx).await.unwrap();
        assert!(ctx.is_filtered);
    }

    #[tokio::test]
    async fn test_level_filter_keeps_at_minimum() {
        let mut stage = LevelFilterStage::new(LogLevel::Warn);
        let mut ctx = make_context(LogLevel::Warn, "mensaje de warn");
        stage.process(&mut ctx).await.unwrap();
        assert!(!ctx.is_filtered);
    }

    #[tokio::test]
    async fn test_field_enricher_adds_fields() {
        let mut fields = HashMap::new();
        fields.insert("env".to_string(), serde_json::json!("production"));
        let mut stage = FieldEnricherStage::new(fields);
        let mut ctx = make_context(LogLevel::Info, "servicio iniciado");
        stage.process(&mut ctx).await.unwrap();
        assert_eq!(
            ctx.entry.fields.get("env"),
            Some(&serde_json::json!("production"))
        );
    }

    #[tokio::test]
    async fn test_field_enricher_skips_filtered() {
        let mut fields = HashMap::new();
        fields.insert("env".to_string(), serde_json::json!("production"));
        let mut stage = FieldEnricherStage::new(fields);
        let mut ctx = make_context(LogLevel::Info, "servicio iniciado");
        ctx.filter();
        stage.process(&mut ctx).await.unwrap();
        assert!(ctx.entry.fields.get("env").is_none());
    }

    #[tokio::test]
    async fn test_keyword_filter_exclude_matching() {
        let mut stage = KeywordFilterStage::new(
            vec!["health".to_string()],
            true,
        );
        let mut ctx = make_context(LogLevel::Info, "health check ok");
        stage.process(&mut ctx).await.unwrap();
        assert!(ctx.is_filtered);
    }

    #[tokio::test]
    async fn test_keyword_filter_include_only_matching() {
        let mut stage = KeywordFilterStage::new(
            vec!["error".to_string()],
            false,
        );
        let mut ctx = make_context(LogLevel::Info, "operacion exitosa");
        stage.process(&mut ctx).await.unwrap();
        assert!(ctx.is_filtered);
    }
}
