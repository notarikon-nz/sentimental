use clap::{Parser, Subcommand};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::{fs};
use thiserror::Error;
use vader_sentiment::SentimentIntensityAnalyzer;

/// Configuration for the sentiment analyzer
#[derive(Debug, Serialize, Deserialize)]
struct Config {
    analysis: AnalysisConfig,
    logging: LoggingConfig,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnalysisConfig {
    positive_threshold: f64,
    negative_threshold: f64,
    include_compound: bool,
    include_individual: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct LoggingConfig {
    level: String,
    file: String,
}

/// Sentiment analysis result
#[derive(Debug, Serialize)]
struct SentimentResult {
    text: String,
    classification: String,
    scores: SentimentScores,
}

#[derive(Debug, Serialize)]
struct SentimentScores {
    compound: Option<f64>,
    positive: Option<f64>,
    negative: Option<f64>,
    neutral: Option<f64>,
}

/// Custom error types for the application
#[derive(Error, Debug)]
enum SentimentError {
    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("YAML parsing error: {0}")]
    YamlError(#[from] serde_yaml::Error),

    #[error("Logging initialization error: {0}")]
    LoggingError(#[from] log::SetLoggerError),
}

/// CLI interface definition
#[derive(Parser, Debug)]
#[command(name = "Sentiment Analyzer")]
#[command(version = "1.0")]
#[command(about = "Analyzes sentiment of text using VADER algorithm", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Analyze sentiment of a single text
    Analyze {
        /// Text to analyze
        text: String,
        
        /// Optional config file path
        #[arg(short, long, default_value = "config.yaml")]
        config: String,
    },
    /// Analyze sentiment of a text file (one sentence per line)
    AnalyzeFile {
        /// Path to text file
        file: String,
        
        /// Optional config file path
        #[arg(short, long, default_value = "config.yaml")]
        config: String,
    },
}

/// Load configuration from YAML file
fn load_config(config_path: &str) -> Result<Config, SentimentError> {
    let config_content = fs::read_to_string(config_path)?;
    let config: Config = serde_yaml::from_str(&config_content)?;
    Ok(config)
}

/// Initialize logging system
fn init_logging(config: &LoggingConfig) -> Result<(), SentimentError> {
    let log_level = match config.level.to_lowercase().as_str() {
        "debug" => log::LevelFilter::Debug,
        "info" => log::LevelFilter::Info,
        "warn" => log::LevelFilter::Warn,
        "error" => log::LevelFilter::Error,
        _ => log::LevelFilter::Info,
    };

    if config.file.is_empty() {
        env_logger::Builder::new()
            .filter_level(log_level)
            .try_init()?;
    } else {
        let log_file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&config.file)?;
            
        env_logger::Builder::new()
            .filter_level(log_level)
            .target(env_logger::Target::Pipe(Box::new(log_file)))
            .try_init()?;
    }

    Ok(())
}

/// Classify sentiment based on thresholds
fn classify_sentiment(score: f64, config: &AnalysisConfig) -> String {
    if score >= config.positive_threshold {
        "Positive".to_string()
    } else if score <= config.negative_threshold {
        "Negative".to_string()
    } else {
        "Neutral".to_string()
    }
}

/// Analyze sentiment of a single text
fn analyze_text(text: &str, config: &AnalysisConfig) -> SentimentResult {
    let analyzer = SentimentIntensityAnalyzer::new();
    let scores = analyzer.polarity_scores(text);

    // Extract scores from the HashMap
    let compound = *scores.get("compound").unwrap_or(&0.0);
    let positive = *scores.get("pos").unwrap_or(&0.0);
    let negative = *scores.get("neg").unwrap_or(&0.0);
    let neutral = *scores.get("neu").unwrap_or(&0.0);

    let classification = classify_sentiment(compound, config);

    SentimentResult {
        text: text.to_string(),
        classification,
        scores: SentimentScores {
            compound: if config.include_compound {
                Some(compound)
            } else {
                None
            },
            positive: if config.include_individual {
                Some(positive)
            } else {
                None
            },
            negative: if config.include_individual {
                Some(negative)
            } else {
                None
            },
            neutral: if config.include_individual {
                Some(neutral)
            } else {
                None
            },
        },
    }
}

/// Process a single text analysis with error handling
fn process_text(text: &str, config_path: &str) -> Result<(), SentimentError> {
    let config = match load_config(config_path) {
        Ok(cfg) => cfg,
        Err(e) => {
            warn!("Failed to load config from '{}': {}. Using defaults.", config_path, e);
            // Fallback to default config
            Config {
                analysis: AnalysisConfig {
                    positive_threshold: 0.05,
                    negative_threshold: -0.05,
                    include_compound: true,
                    include_individual: false,
                },
                logging: LoggingConfig {
                    level: "info".to_string(),
                    file: "".to_string(),
                },
            }
        }
    };

    // Initialize logging (fallback to console if config loading failed)
    if let Err(e) = init_logging(&config.logging) {
        eprintln!("Failed to initialize logging: {}. Using console only.", e);
        env_logger::Builder::new()
            .filter_level(log::LevelFilter::Info)
            .try_init()
            .map_err(SentimentError::LoggingError)?;
    }

    info!("Analyzing text: '{}'", text);
    let result = analyze_text(text, &config.analysis);
    
    println!("\nSentiment Analysis Result:");
    println!("Text: {}", result.text);
    println!("Classification: {}", result.classification);
    
    if let Some(compound) = result.scores.compound {
        println!("Compound Score: {:.4}", compound);
    }
    
    if config.analysis.include_individual {
        if let Some(pos) = result.scores.positive {
            println!("Positive: {:.4}", pos);
        }
        if let Some(neg) = result.scores.negative {
            println!("Negative: {:.4}", neg);
        }
        if let Some(neu) = result.scores.neutral {
            println!("Neutral: {:.4}", neu);
        }
    }

    Ok(())
}

/// Process a file with multiple texts (one per line)
fn process_file(file_path: &str, config_path: &str) -> Result<(), SentimentError> {
    let config = match load_config(config_path) {
        Ok(cfg) => cfg,
        Err(e) => {
            warn!("Failed to load config from '{}': {}. Using defaults.", config_path, e);
            Config {
                analysis: AnalysisConfig {
                    positive_threshold: 0.05,
                    negative_threshold: -0.05,
                    include_compound: true,
                    include_individual: false,
                },
                logging: LoggingConfig {
                    level: "info".to_string(),
                    file: "".to_string(),
                },
            }
        }
    };

    if let Err(e) = init_logging(&config.logging) {
        eprintln!("Failed to initialize logging: {}. Using console only.", e);
        env_logger::Builder::new()
            .filter_level(log::LevelFilter::Info)
            .try_init()
            .map_err(SentimentError::LoggingError)?;
    }

    info!("Processing file: {}", file_path);
    let content = fs::read_to_string(file_path)?;
    let lines = content.lines().filter(|l| !l.trim().is_empty());

    println!("File Analysis Results:");
    println!("=====================");

    for (i, line) in lines.enumerate() {
        match analyze_text(line, &config.analysis) {
            result => {
                println!("\nLine {}:", i + 1);
                println!("Text: {}", result.text);
                println!("Classification: {}", result.classification);
                
                if let Some(compound) = result.scores.compound {
                    println!("Compound Score: {:.4}", compound);
                }
                
                if config.analysis.include_individual {
                    if let Some(pos) = result.scores.positive {
                        println!("Positive: {:.4}", pos);
                    }
                    if let Some(neg) = result.scores.negative {
                        println!("Negative: {:.4}", neg);
                    }
                    if let Some(neu) = result.scores.neutral {
                        println!("Neutral: {:.4}", neu);
                    }
                }
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Analyze { text, config } => process_text(&text, &config),
        Commands::AnalyzeFile { file, config } => process_file(&file, &config),
    };

    if let Err(e) = result {
        error!("Application error: {}", e);
        std::process::exit(1);
    }
}