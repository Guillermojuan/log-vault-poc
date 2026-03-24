pub mod args;
pub use args::Args;

use clap::Parser as ClapParser;
use std::collections::HashMap;

use crate::application::stages::{FieldEnricherStage, KeywordFilterStage, LevelFilterStage};
use crate::application::Pipeline;
use crate::domain::Middleware;
use crate::infrastructure::{FileReader, JsonFileWriter, JsonParser, PlainTextParser};

use args::CliLogFormat;

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let parser: Box<dyn crate::domain::Parser<crate::domain::LogEntry>> = match args.format {
        CliLogFormat::Json  => Box::new(JsonParser),
        CliLogFormat::Plain => Box::new(PlainTextParser::new()?),
    };

    let reader = FileReader::new(args.input, parser);
    let entries = reader.read_all().await?;
    let total_read = entries.len();

    let minimum_level = args.level.into();
    let mut stages: Vec<Box<dyn Middleware>> = vec![
        Box::new(LevelFilterStage::new(minimum_level)),
    ];

    if !args.include.is_empty() {
        stages.push(Box::new(KeywordFilterStage::new(args.include, false)));
    }

    if !args.exclude.is_empty() {
        stages.push(Box::new(KeywordFilterStage::new(args.exclude, true)));
    }

    if !args.enrich.is_empty() {
        let mut fields = HashMap::new();
        for pair in &args.enrich {
            let mut parts = pair.splitn(2, '=');
            let key = parts.next().unwrap_or("").to_string();
            let val = parts.next().unwrap_or("").to_string();
            if !key.is_empty() {
                fields.insert(key, serde_json::Value::String(val));
            }
        }
        stages.push(Box::new(FieldEnricherStage::new(fields)));
    }

    let sink = Box::new(JsonFileWriter::new(&args.output).await?);
    let mut pipeline = Pipeline::new(stages, sink);
    let stats = pipeline.run(entries).await?;

    println!("log-vault completado.");
    println!("  Entradas leidas : {}", total_read);
    println!("  {}", stats);

    Ok(())
}
