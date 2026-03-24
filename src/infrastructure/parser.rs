use async_trait::async_trait;
use regex::Regex;
use chrono::DateTime;

use crate::domain::{DomainError, LogEntry, LogLevel, Parser};

pub struct JsonParser;

#[async_trait]
impl Parser<LogEntry> for JsonParser {
    async fn parse(&self, raw: &str) -> Result<LogEntry, DomainError> {
        serde_json::from_str::<LogEntry>(raw).map_err(|e| DomainError::ParseError {
            format: "json".to_string(),
            message: e.to_string(),
        })
    }
}

pub struct PlainTextParser {
    pattern: Regex,
}

impl PlainTextParser {
    pub fn new() -> Result<Self, DomainError> {
        let pattern = Regex::new(
            r"^(?P<timestamp>\S+)\s+(?P<level>TRACE|DEBUG|INFO|WARN|ERROR|FATAL)\s+(?:\[(?P<source>[^\]]+)\]\s+)?(?P<message>.+)$"
        )
        .map_err(|e| DomainError::ParseError {
            format: "plaintext".to_string(),
            message: e.to_string(),
        })?;

        Ok(Self { pattern })
    }
}

#[async_trait]
impl Parser<LogEntry> for PlainTextParser {
    async fn parse(&self, raw: &str) -> Result<LogEntry, DomainError> {
        let caps = self.pattern.captures(raw).ok_or_else(|| DomainError::ParseError {
            format: "plaintext".to_string(),
            message: format!("Line does not match expected pattern: '{}'", raw),
        })?;

        let timestamp_str = &caps["timestamp"];
        let timestamp = DateTime::parse_from_rfc3339(timestamp_str)
            .map_err(|e| DomainError::ParseError {
                format: "plaintext".to_string(),
                message: format!("Invalid timestamp '{}': {}", timestamp_str, e),
            })?
            .with_timezone(&chrono::Utc);

        let level = parse_level(&caps["level"])?;
        let message = caps["message"].to_string();
        let source = caps.name("source").map(|m| m.as_str().to_string());

        let mut entry = LogEntry::new(timestamp, level, message);
        entry.source = source;

        Ok(entry)
    }
}

fn parse_level(raw: &str) -> Result<LogLevel, DomainError> {
    match raw {
        "TRACE" => Ok(LogLevel::Trace),
        "DEBUG" => Ok(LogLevel::Debug),
        "INFO"  => Ok(LogLevel::Info),
        "WARN"  => Ok(LogLevel::Warn),
        "ERROR" => Ok(LogLevel::Error),
        "FATAL" => Ok(LogLevel::Fatal),
        other   => Err(DomainError::InvalidLogLevel {
            value: other.to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_json_parser_valid() {
        let parser = JsonParser;
        let raw = r#"{
            "timestamp": "2026-03-24T10:15:30Z",
            "level": "Info",
            "message": "Usuario autenticado",
            "source": "AuthService",
            "fields": {}
        }"#;
        let result = parser.parse(raw).await;
        assert!(result.is_ok());
        let entry = result.unwrap();
        assert_eq!(entry.level, LogLevel::Info);
        assert_eq!(entry.source, Some("AuthService".to_string()));
    }

    #[tokio::test]
    async fn test_plaintext_parser_valid() {
        let parser = PlainTextParser::new().unwrap();
        let raw = "2026-03-24T10:15:30Z INFO [AuthService] Usuario autenticado";
        let result = parser.parse(raw).await;
        assert!(result.is_ok());
        let entry = result.unwrap();
        assert_eq!(entry.level, LogLevel::Info);
        assert_eq!(entry.message, "Usuario autenticado");
        assert_eq!(entry.source, Some("AuthService".to_string()));
    }

    #[tokio::test]
    async fn test_plaintext_parser_without_source() {
        let parser = PlainTextParser::new().unwrap();
        let raw = "2026-03-24T10:15:30Z WARN Memoria disponible por debajo del umbral";
        let result = parser.parse(raw).await;
        assert!(result.is_ok());
        let entry = result.unwrap();
        assert_eq!(entry.level, LogLevel::Warn);
        assert!(entry.source.is_none());
    }

    #[tokio::test]
    async fn test_plaintext_parser_invalid_line() {
        let parser = PlainTextParser::new().unwrap();
        let raw = "esto no es un log valido";
        let result = parser.parse(raw).await;
        assert!(result.is_err());
    }
}
