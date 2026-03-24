use std::path::PathBuf;
use clap::{Parser, ValueEnum};

#[derive(Debug, Parser)]
#[command(
    name = "log-vault",
    version = "0.1.0",
    about = "Procesa y filtra archivos de log con un pipeline de middlewares"
)]
pub struct Args {
    #[arg(
        short,
        long,
        help = "Archivo de log de entrada"
    )]
    pub input: PathBuf,

    #[arg(
        short,
        long,
        help = "Archivo JSON de salida"
    )]
    pub output: PathBuf,

    #[arg(
        short,
        long,
        value_enum,
        help = "Formato del archivo de entrada"
    )]
    pub format: CliLogFormat,

    #[arg(
        short,
        long,
        value_enum,
        default_value = "info",
        help = "Nivel mínimo de log a procesar"
    )]
    pub level: CliLogLevel,

    #[arg(
        long,
        value_delimiter = ',',
        help = "Keywords a incluir (separa con comas)"
    )]
    pub include: Vec<String>,

    #[arg(
        long,
        value_delimiter = ',',
        help = "Keywords a excluir (separa con comas)"
    )]
    pub exclude: Vec<String>,

    #[arg(
        long,
        value_delimiter = ',',
        help = "Campos extra a enriquecer en formato key=value"
    )]
    pub enrich: Vec<String>,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum CliLogFormat {
    Json,
    Plain,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum CliLogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

impl From<CliLogLevel> for crate::domain::LogLevel {
    fn from(val: CliLogLevel) -> Self {
        match val {
            CliLogLevel::Trace => crate::domain::LogLevel::Trace,
            CliLogLevel::Debug => crate::domain::LogLevel::Debug,
            CliLogLevel::Info  => crate::domain::LogLevel::Info,
            CliLogLevel::Warn  => crate::domain::LogLevel::Warn,
            CliLogLevel::Error => crate::domain::LogLevel::Error,
            CliLogLevel::Fatal => crate::domain::LogLevel::Fatal,
        }
    }
}
