//! paper-bench — Research benchmark harness for self-play fine-tuning paper.
//!
//! Scores code generation models on the Opus-201 benchmark (201 Rust problems,
//! 6 difficulty tiers, verified by cargo test) and HumanEval-Rust.
//!
//! Usage:
//!   paper-bench score-claude --problems 5          # smoke test Claude
//!   paper-bench score-gemini --problems 5          # smoke test Gemini
//!   paper-bench score-local --model models/qwen.gguf --problems 5
//!   paper-bench selfplay --model models/qwen.gguf --iterations 10
//!   paper-bench summary                             # compare all results

mod backends;
mod humaneval;
mod results;
mod runner;
mod selfplay;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "paper-bench", about = "Benchmark code models for self-play fine-tuning research")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Score Claude Opus on the benchmark (ceiling)
    ScoreClaude {
        #[arg(long, env = "ANTHROPIC_API_KEY")]
        api_key: String,
        #[arg(long, default_value = "claude-opus-4-6-20260411")]
        model: String,
        /// Max problems (0 = all 201)
        #[arg(long, default_value = "0")]
        problems: usize,
        #[arg(long, default_value = "results/claude-opus-4.json")]
        output: String,
        /// Also run HumanEval-Rust
        #[arg(long)]
        humaneval: bool,
    },
    /// Score Gemini Flash Lite (baseline)
    ScoreGemini {
        #[arg(long, env = "GEMINI_API_KEY")]
        api_key: String,
        #[arg(long, default_value = "gemini-2.5-flash-lite-preview-06-17")]
        model: String,
        #[arg(long, default_value = "0")]
        problems: usize,
        #[arg(long, default_value = "results/gemini-flash-lite.json")]
        output: String,
        #[arg(long)]
        humaneval: bool,
    },
    /// Score a local GGUF model (Qwen, DeepSeek, etc.)
    ScoreLocal {
        /// Path to GGUF model file (for CLI mode)
        #[arg(long, default_value = "")]
        model: String,
        /// URL of running llama-server (faster — model stays loaded)
        #[arg(long)]
        server_url: Option<String>,
        /// Human-readable model name for results
        #[arg(long, default_value = "local-model")]
        name: String,
        #[arg(long, default_value = "0")]
        problems: usize,
        #[arg(long, default_value = "results/local-model.json")]
        output: String,
        #[arg(long)]
        humaneval: bool,
        /// Best-of-N: attempt each problem N times, take first pass
        #[arg(long, default_value = "1")]
        samples: usize,
        /// Override temperature for inference
        #[arg(long)]
        temperature: Option<f32>,
    },
    /// Run self-play fine-tuning loop
    Selfplay {
        /// Path to GGUF model file (or API backend)
        #[arg(long)]
        model: Option<String>,
        /// URL of running llama-server (faster — model stays loaded)
        #[arg(long)]
        server_url: Option<String>,
        /// Use Claude API instead of local model
        #[arg(long, env = "ANTHROPIC_API_KEY")]
        claude_key: Option<String>,
        /// Use Gemini API instead of local model
        #[arg(long, env = "GEMINI_API_KEY")]
        gemini_key: Option<String>,
        /// Number of self-play iterations
        #[arg(long, default_value = "10")]
        iterations: usize,
        /// Max problems per iteration (0 = all)
        #[arg(long, default_value = "0")]
        problems: usize,
        /// Output directory
        #[arg(long, default_value = "selfplay_runs")]
        output_dir: String,
    },
    /// Score a local fine-tuned model via native GPU inference (candle).
    /// Loads safetensors + tokenizer from an HF-style directory; no llama-server required.
    ScoreCandle {
        /// Path to a local model directory containing model.safetensors, tokenizer.json, config.json
        #[arg(long)]
        model_dir: String,
        /// Human-readable model name for results
        #[arg(long, default_value = "local-candle")]
        name: String,
        #[arg(long, default_value = "0")]
        problems: usize,
        #[arg(long, default_value = "results/local-candle.json")]
        output: String,
        /// 0.0 = greedy. Typical pass@5 runs: 0.7.
        #[arg(long, default_value = "0.0")]
        temperature: f32,
        /// Best-of-N sampling (requires temperature > 0).
        #[arg(long, default_value = "1")]
        samples: usize,
        /// Max new tokens generated per problem.
        #[arg(long, default_value = "1024")]
        max_tokens: usize,
    },
    /// Fetch HumanEval-Rust from HuggingFace
    FetchHumaneval,
    /// Show summary of all results
    Summary {
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
            humaneval,
        } => {
            let generator = backends::claude::ClaudeGenerator::new(api_key, model);
            let limit = if problems == 0 { None } else { Some(problems) };
            runner::run_benchmark(&generator, limit, &output).await;
            if humaneval {
                let he_problems = humaneval::load_humaneval_problems();
                let he_output = output.replace(".json", "-humaneval.json");
                runner::run_benchmark_on(&generator, &he_problems, limit, &he_output).await;
            }
        }
        Command::ScoreGemini {
            api_key,
            model,
            problems,
            output,
            humaneval,
        } => {
            let generator = backends::gemini::GeminiGenerator::new(api_key, model);
            let limit = if problems == 0 { None } else { Some(problems) };
            runner::run_benchmark(&generator, limit, &output).await;
            if humaneval {
                let he_problems = humaneval::load_humaneval_problems();
                let he_output = output.replace(".json", "-humaneval.json");
                runner::run_benchmark_on(&generator, &he_problems, limit, &he_output).await;
            }
        }
        Command::ScoreLocal {
            model,
            server_url,
            name,
            problems,
            output,
            humaneval,
            samples,
            temperature,
        } => {
            let mut generator = backends::local::LocalModelGenerator::new(name, model);
            if let Some(url) = server_url {
                generator = generator.with_server(url);
            }
            if let Some(temp) = temperature {
                generator = generator.with_temperature(temp);
            }
            let limit = if problems == 0 { None } else { Some(problems) };
            let all_problems = x402_soul::opus_bench::load_embedded_problems();
            runner::run_benchmark_on_with_samples(&generator, &all_problems, limit, &output, samples).await;
            if humaneval {
                let he_problems = humaneval::load_humaneval_problems();
                let he_output = output.replace(".json", "-humaneval.json");
                runner::run_benchmark_on(&generator, &he_problems, limit, &he_output).await;
            }
        }
        Command::Selfplay {
            model,
            server_url,
            claude_key,
            gemini_key,
            iterations,
            problems,
            output_dir,
        } => {
            let generator: Box<dyn runner::CodeGenerator> = if let Some(url) = server_url {
                let mut gen = backends::local::LocalModelGenerator::new(
                    "qwen-selfplay".to_string(),
                    String::new(),
                );
                gen = gen.with_server(url);
                Box::new(gen)
            } else if let Some(path) = model {
                Box::new(backends::local::LocalModelGenerator::new(
                    "qwen-selfplay".to_string(),
                    path,
                ))
            } else if let Some(key) = claude_key {
                Box::new(backends::claude::ClaudeGenerator::new(
                    key,
                    "claude-opus-4-6-20260411".to_string(),
                ))
            } else if let Some(key) = gemini_key {
                Box::new(backends::gemini::GeminiGenerator::new(
                    key,
                    "gemini-2.5-flash-lite-preview-06-17".to_string(),
                ))
            } else {
                eprintln!("Error: specify --model (GGUF path), --claude-key, or --gemini-key");
                std::process::exit(1);
            };

            let all_problems = x402_soul::opus_bench::load_embedded_problems();
            let config = selfplay::SelfPlayConfig {
                iterations,
                max_problems: problems,
                output_dir,
                ..Default::default()
            };

            selfplay::run_selfplay(generator.as_ref(), &all_problems, &config).await;
        }
        Command::ScoreCandle {
            model_dir,
            name,
            problems,
            output,
            temperature,
            samples,
            max_tokens,
        } => {
            let generator = backends::candle::CandleGenerator::from_dir(
                name,
                &model_dir,
                temperature as f64,
            )
            .unwrap_or_else(|e| {
                eprintln!("candle: failed to load model: {e:#}");
                std::process::exit(1);
            })
            .with_max_tokens(max_tokens);

            let limit = if problems == 0 { None } else { Some(problems) };
            let all_problems = x402_soul::opus_bench::load_embedded_problems();
            runner::run_benchmark_on_with_samples(
                &generator,
                &all_problems,
                limit,
                &output,
                samples,
            )
            .await;
        }
        Command::FetchHumaneval => match humaneval::fetch_and_cache().await {
            Ok(count) => println!("Fetched {count} HumanEval-Rust problems"),
            Err(e) => eprintln!("Error: {e}"),
        },
        Command::Summary { dir } => {
            results::print_summary(&dir);
        }
    }
}
