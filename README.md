# Sentiment Analyzer

A minimal Rust CLI tool for text sentiment analysis using VADER. Handles large files efficiently with streaming.

## Usage

```bash
# Analyze single text
cargo run -- "I love Rust!"

# Analyze text file (line by line)
cargo run -- --file input.txt

# Custom thresholds and verbose output
cargo run -- --file data.txt --pos-threshold 0.1 --neg-threshold -0.1 --verbose
```

## Options

- `--file`: Treat input as file path
- `--pos-threshold`: Positive sentiment threshold (default: 0.05)  
- `--neg-threshold`: Negative sentiment threshold (default: -0.05)
- `--verbose`: Show detailed pos/neg/neu scores

## Safety Features

- Memory efficient: Streams large files line-by-line (constant ~8KB memory)
- Error resilient: Continues processing if individual lines fail
- Input validation: Checks thresholds and handles empty lines

## Known Limitations & Risks

- No interrupt handling: Large file processing can't be stopped with Ctrl+C gracefully
- Unicode panic risk: Very long lines with Unicode may cause crashes on text truncation
- No processing limits: Malicious files with millions of lines could run indefinitely  
- Path traversal: No validation prevents access to system files (`--file ../../../etc/passwd`)
- Error spam: Files with many corrupted lines will flood stderr with no rate limiting
