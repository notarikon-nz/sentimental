use clap::Parser;
use std::fs;
use vader_sentiment::SentimentIntensityAnalyzer;

#[derive(Parser)]
#[command(name = "sentiment", version = "1.0")]
struct Cli {
    /// Text to analyze or file path (use --file for files)
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

fn classify(score: f64, pos_thresh: f64, neg_thresh: f64) -> &'static str {
    if score >= pos_thresh { "Positive" }
    else if score <= neg_thresh { "Negative" }
    else { "Neutral" }
}

fn analyze_text(text: &str, cli: &Cli) {
    let analyzer = SentimentIntensityAnalyzer::new();
    let scores = analyzer.polarity_scores(text);
    let compound = scores["compound"];
    
    println!("{}: {} ({})", 
        classify(compound, cli.pos_threshold, cli.neg_threshold),
        text.chars().take(50).collect::<String>(),
        compound
    );
    
    if cli.verbose {
        println!("  pos: {}, neg: {}, neu: {}", 
            scores["pos"], scores["neg"], scores["neu"]);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    if cli.file {
        for (i, line) in fs::read_to_string(&cli.input)?.lines().enumerate() {
            if !line.trim().is_empty() {
                print!("Line {}: ", i + 1);
                analyze_text(line, &cli);
            }
        }
    } else {
        analyze_text(&cli.input, &cli);
    }
    
    Ok(())
}