use clap::Parser;
use std::fs::File;
use std::io::{BufRead, BufReader};
use vader_sentiment::SentimentIntensityAnalyzer;

#[derive(Parser)]
#[command(name = "sentiment", version = "1.0")]
struct Cli {
    input: String,
    #[arg(long)]
    file: bool,
    #[arg(long, default_value = "0.05")]
    pos_threshold: f64,
    #[arg(long, default_value = "-0.05")]  
    neg_threshold: f64,
    #[arg(short, long)]
    verbose: bool,
}

fn classify(score: f64, pos: f64, neg: f64) -> &'static str {
    if score >= pos { "Positive" }
    else if score <= neg { "Negative" }
    else { "Neutral" }
}

fn analyze(text: &str, analyzer: &SentimentIntensityAnalyzer, cli: &Cli) {
    if text.trim().is_empty() { return; }
    
    let scores = analyzer.polarity_scores(text);
    let compound = scores["compound"];
    
    let display = if text.len() > 60 { &text[..57] } else { text };
    println!("{}: {} ({:.3})", 
        classify(compound, cli.pos_threshold, cli.neg_threshold), 
        display, compound);
    
    if cli.verbose {
        println!("  pos: {:.3}, neg: {:.3}, neu: {:.3}", 
            scores["pos"], scores["neg"], scores["neu"]);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    if cli.pos_threshold <= cli.neg_threshold {
        eprintln!("Error: Positive threshold must be greater than negative threshold");
        std::process::exit(1);
    }
    
    let analyzer = SentimentIntensityAnalyzer::new();
    
    if cli.file {
        let file = File::open(&cli.input)?;
        let reader = BufReader::new(file);
        
        for (i, line) in reader.lines().enumerate() {
            match line {
                Ok(text) => {
                    print!("Line {}: ", i + 1);
                    analyze(&text, &analyzer, &cli);
                }
                Err(e) => eprintln!("Line {}: Error reading - {}", i + 1, e),
            }
        }
    } else {
        analyze(&cli.input, &analyzer, &cli);
    }
    
    Ok(())
}
