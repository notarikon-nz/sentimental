# Sentiment Analyzer

A Rust CLI tool for text sentiment analysis using VADER.

## Features

- Analyze text sentiment (Positive/Negative/Neutral)
- Process text files (one entry per line)
- Configurable via `config.yaml`
- Logging to file or console

## Usage

```bash
# Analyze single text
cargo run -- analyze "I love Rust!"

# Analyze text file
cargo run -- analyze-file input.txt

# Show help
cargo run -- --help