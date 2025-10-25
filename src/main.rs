use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use prompt_compress::{
    init_optimizer, load_corpus, save_corpus, DirectiveFormat, Language, OptimizationRequest,
};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "prompt-compress")]
#[command(about = "Optimize prompts with multilingual token compression", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Optimize a prompt
    Optimize {
        /// Input file containing the prompt
        #[arg(short, long)]
        input: PathBuf,

        /// Output file for optimized prompt
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Output language (english or mandarin)
        #[arg(long, default_value = "english")]
        output_lang: String,

        /// Confidence threshold (0.0-1.0)
        #[arg(long, default_value = "0.85")]
        threshold: f64,

        /// Aggressive mode (lower threshold)
        #[arg(long)]
        aggressive: bool,

        /// Directive format (bracketed, instructive, xml, natural)
        #[arg(long, default_value = "bracketed")]
        directive_format: String,

        /// Interactive mode for HITL review
        #[arg(long)]
        interactive: bool,
    },

    /// Analyze prompt without optimizing
    Analyze {
        /// Input file containing the prompt
        #[arg(short, long)]
        input: PathBuf,

        /// Report output file
        #[arg(short, long)]
        report: Option<PathBuf>,
    },

    /// Update priors from feedback
    Train {
        /// Feedback file (JSON)
        #[arg(short, long)]
        feedback: PathBuf,

        /// Corpus file to update
        #[arg(short, long, default_value = "data/priors.json")]
        corpus: PathBuf,
    },

    /// Batch process multiple prompts
    Batch {
        /// Input directory
        #[arg(short, long)]
        input: PathBuf,

        /// Output directory
        #[arg(short, long)]
        output: PathBuf,

        /// Output language (english or mandarin)
        #[arg(long, default_value = "english")]
        output_lang: String,
    },
}

fn main() -> Result<()> {
    env_logger::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Optimize {
            input,
            output,
            output_lang,
            threshold,
            aggressive,
            directive_format,
            interactive,
        } => {
            optimize_command(
                input,
                output,
                output_lang,
                threshold,
                aggressive,
                directive_format,
                interactive,
            )?;
        }
        Commands::Analyze { input, report } => {
            analyze_command(input, report)?;
        }
        Commands::Train { feedback, corpus } => {
            train_command(feedback, corpus)?;
        }
        Commands::Batch {
            input,
            output,
            output_lang,
        } => {
            batch_command(input, output, output_lang)?;
        }
    }

    Ok(())
}

fn optimize_command(
    input: PathBuf,
    output: Option<PathBuf>,
    output_lang: String,
    threshold: f64,
    aggressive: bool,
    directive_format: String,
    interactive: bool,
) -> Result<()> {
    let prompt = std::fs::read_to_string(&input)
        .with_context(|| format!("Failed to read input file: {:?}", input))?;

    let language = match output_lang.to_lowercase().as_str() {
        "mandarin" | "zh" => Language::Mandarin,
        _ => Language::English,
    };

    let format = match directive_format.to_lowercase().as_str() {
        "instructive" => DirectiveFormat::Instructive,
        "xml" => DirectiveFormat::Xml,
        "natural" => DirectiveFormat::Natural,
        _ => DirectiveFormat::Bracketed,
    };

    let request = OptimizationRequest {
        prompt,
        output_language: language,
        confidence_threshold: threshold,
        aggressive_mode: aggressive,
        directive_format: format,
    };

    let mut optimizer = init_optimizer()?;
    let result = optimizer.optimize(&request)?;

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Optimization complete!");
    println!();
    println!("Original: {} tokens", result.original_tokens);
    println!("Optimized: {} tokens", result.optimized_tokens);
    println!(
        "Savings: {} tokens ({:.1}%)",
        result.token_savings, result.savings_percentage
    );
    println!();
    println!(
        "Auto-applied optimizations: {}",
        result.optimizations.len()
    );
    println!(
        "Requires review: {}",
        result.requires_review.len()
    );
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    if interactive && !result.requires_review.is_empty() {
        println!("\nReview mode not yet implemented in CLI");
        println!("Use the API server for interactive review");
    }

    if let Some(output_path) = output {
        std::fs::write(&output_path, &result.optimized_prompt)
            .with_context(|| format!("Failed to write output file: {:?}", output_path))?;
        println!("\nOptimized prompt saved to: {:?}", output_path);
    } else {
        println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Optimized Prompt:");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("{}", result.optimized_prompt);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    }

    Ok(())
}

fn analyze_command(input: PathBuf, report: Option<PathBuf>) -> Result<()> {
    let prompt = std::fs::read_to_string(&input)
        .with_context(|| format!("Failed to read input file: {:?}", input))?;

    let request = OptimizationRequest {
        prompt,
        output_language: Language::English,
        confidence_threshold: 0.85,
        aggressive_mode: false,
        directive_format: DirectiveFormat::Bracketed,
    };

    let mut optimizer = init_optimizer()?;
    let result = optimizer.optimize(&request)?;

    let analysis = serde_json::json!({
        "original_tokens": result.original_tokens,
        "potential_savings": result.token_savings,
        "savings_percentage": result.savings_percentage,
        "optimizations": result.optimizations,
        "requires_review": result.requires_review,
    });

    if let Some(report_path) = report {
        let json = serde_json::to_string_pretty(&analysis)?;
        std::fs::write(&report_path, json)
            .with_context(|| format!("Failed to write report: {:?}", report_path))?;
        println!("Analysis report saved to: {:?}", report_path);
    } else {
        println!("{}", serde_json::to_string_pretty(&analysis)?);
    }

    Ok(())
}

fn train_command(_feedback: PathBuf, _corpus: PathBuf) -> Result<()> {
    println!("Training from feedback not yet implemented");
    println!("Use the API server for interactive training");
    Ok(())
}

fn batch_command(input: PathBuf, output: PathBuf, output_lang: String) -> Result<()> {
    if !input.is_dir() {
        anyhow::bail!("Input must be a directory");
    }

    std::fs::create_dir_all(&output)
        .with_context(|| format!("Failed to create output directory: {:?}", output))?;

    let language = match output_lang.to_lowercase().as_str() {
        "mandarin" | "zh" => Language::Mandarin,
        _ => Language::English,
    };

    let mut optimizer = init_optimizer()?;
    let mut total_processed = 0;
    let mut total_savings = 0i64;

    for entry in std::fs::read_dir(&input)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("txt") {
            let prompt = std::fs::read_to_string(&path)?;

            let request = OptimizationRequest {
                prompt,
                output_language: language.clone(),
                confidence_threshold: 0.85,
                aggressive_mode: false,
                directive_format: DirectiveFormat::Bracketed,
            };

            match optimizer.optimize(&request) {
                Ok(result) => {
                    let output_file = output.join(path.file_name().unwrap());
                    std::fs::write(&output_file, &result.optimized_prompt)?;

                    total_processed += 1;
                    total_savings += result.token_savings;

                    println!(
                        "✓ {:?}: {} tokens saved ({:.1}%)",
                        path.file_name().unwrap(),
                        result.token_savings,
                        result.savings_percentage
                    );
                }
                Err(e) => {
                    eprintln!("✗ {:?}: {}", path.file_name().unwrap(), e);
                }
            }
        }
    }

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Batch processing complete!");
    println!("Files processed: {}", total_processed);
    println!("Total tokens saved: {}", total_savings);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    Ok(())
}
