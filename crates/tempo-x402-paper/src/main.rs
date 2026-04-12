//! paper-bench — Research benchmark harness for self-play fine-tuning paper.
//!
//! Scores code generation models on the Opus-201 benchmark (201 Rust problems,
//! 6 difficulty tiers, verified by cargo test).

mod backends;
mod results;
mod runner;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "paper-bench", about = "Benchmark code generation models")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Score Claude Opus on the Opus-201 benchmark
    ScoreClaude {
        /// Anthropic API key (or set ANTHROPIC_API_KEY env var)
        #[arg(long, env = "ANTHROPIC_API_KEY")]
        api_key: String,
        /// Model ID
        #[arg(long, default_value = "claude-opus-4-6-20260411")]
        model: String,
        /// Max problems to run (0 = all)
        #[arg(long, default_value = "0")]
        problems: usize,
        /// Output file
        #[arg(long, default_value = "results/claude-opus-4.json")]
        output: String,
    },
    /// Score Gemini Flash Lite on the Opus-201 benchmark
    ScoreGemini {
        /// Gemini API key (or set GEMINI_API_KEY env var)
        #[arg(long, env = "GEMINI_API_KEY")]
        api_key: String,
        /// Model ID
        #[arg(long, default_value = "gemini-2.5-flash-lite-preview-06-17")]
        model: String,
        /// Max problems to run (0 = all)
        #[arg(long, default_value = "0")]
        problems: usize,
        /// Output file
        #[arg(long, default_value = "results/gemini-flash-lite.json")]
        output: String,
    },
    /// Show summary of existing results
    Summary {
        /// Results directory
        #[arg(long, default_value = "results")]
        dir: String,
    },
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "paper_bench=info".parse().unwrap()),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Command::ScoreClaude {
            api_key,
            model,
            problems,
            output,
        } => {
            let generator = backends::claude::ClaudeGenerator::new(api_key, model);
            let limit = if problems == 0 { None } else { Some(problems) };
            runner::run_benchmark(&generator, limit, &output).await;
        }
        Command::ScoreGemini {
            api_key,
            model,
            problems,
            output,
        } => {
            let generator = backends::gemini::GeminiGenerator::new(api_key, model);
            let limit = if problems == 0 { None } else { Some(problems) };
            runner::run_benchmark(&generator, limit, &output).await;
        }
        Command::Summary { dir } => {
            results::print_summary(&dir);
        }
    }
}
