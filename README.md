# log-vault

CLI tool for processing and filtering log files through a configurable
middleware pipeline.

Built in early 2026 as a rewrite of an internal .NET utility I originally
wrote in 2025. I took it as an opportunity to re-think every architectural
decision in Rust without a framework to lean on — and to have a concrete,
end-to-end project to showcase my systems programming skills.

## What it does

Reads a log file (JSON or plain text), runs each entry through a
pipeline of middleware stages, and writes the output to a structured
JSON file.

Stages: level filtering, keyword include/exclude, field enrichment.

## Architecture

Four layers, strictly unidirectional dependencies:

- domain/         Core types and trait contracts
- application/    Pipeline engine and middleware stages
- infrastructure/ Async I/O, parsers, JSON writer
- cli/            Argument parsing and composition root

No DI container. All composition happens explicitly in cli::run().
Every dependency is visible and intentional.

## Usage

    log-vault \
      --input app.log \
      --output filtered.json \
      --format plain \
      --level warn \
      --exclude health \
      --enrich env=production

| Flag        | Description                      |
|-------------|----------------------------------|
| --input     | Input log file                   |
| --output    | Output JSON file                 |
| --format    | json or plain                    |
| --level     | Minimum level (default: info)    |
| --include   | Keywords to keep                 |
| --exclude   | Keywords to discard              |
| --enrich    | key=value fields to attach       |

## Stack

tokio, serde_json, chrono, thiserror, clap, async-trait, regex

---
Rust Edition 2021 - Stable 1.85 - March 2026