//! `rust-agent` — minimal self-correcting Rust coder using a local candle model.
//!
//! The loop:
//!   1. Read a task spec (problem instructions + starter code + tests).
//!   2. Generate a candidate solution with candle on GPU.
//!   3. Drop it into a scratch Cargo crate, run `cargo check` then `cargo test`.
//!   4. If it fails, feed the diagnostics back to the model and retry (up to N times).
//!   5. Print the final passing solution (or the last attempt with errors).
//!
//! This is the minimal viable loop for a "local-Claude-Code-for-Rust" agent.
//! It reuses the same `CandleGenerator` the benchmark harness uses, so any
//! improvement to the benchmark pipeline automatically benefits the agent.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, Context, Result};
use clap::Parser;

use tempo_x402_paper::backends::candle::CandleGenerator;
use tempo_x402_paper::runner::CodeGenerator;
use x402_soul::benchmark::BenchmarkProblem;

#[derive(Parser)]
#[command(name = "rust-agent", about = "Self-correcting Rust coder using a local candle model")]
struct Args {
    /// HF-style model directory (contains model.safetensors, tokenizer.json, config.json).
    #[arg(long)]
    model_dir: String,

    /// Path to a JSON task spec with fields: slug, instructions, starter_code, test_code, cargo_toml.
    /// Alternatively pass --problem-slug to load an Opus-201 problem by name.
    #[arg(long)]
    task_file: Option<String>,

    /// Load one of the 201 embedded Opus benchmark problems by slug.
    #[arg(long)]
    problem_slug: Option<String>,

    /// Max correction rounds (including the first generation).
    #[arg(long, default_value = "3")]
    max_rounds: usize,

    /// Sampling temperature. 0.0 = greedy.
    #[arg(long, default_value = "0.2")]
    temperature: f32,

    /// Max tokens per generation.
    #[arg(long, default_value = "1024")]
    max_tokens: usize,

    /// If set, write the final solution to this file.
    #[arg(long)]
    output: Option<String>,

    /// Keep the scratch Cargo project (default: remove on exit).
    #[arg(long)]
    keep_workdir: bool,
}

#[derive(Debug)]
struct Task {
    slug: String,
    instructions: String,
    starter_code: String,
    test_code: String,
    cargo_toml: String,
}

impl From<BenchmarkProblem> for Task {
    fn from(p: BenchmarkProblem) -> Self {
        Task {
            slug: p.slug,
            instructions: p.instructions,
            starter_code: p.starter_code,
            test_code: p.test_code,
            cargo_toml: p.cargo_toml,
        }
    }
}

fn load_task(args: &Args) -> Result<Task> {
    if let Some(path) = &args.task_file {
        let raw = fs::read_to_string(path)
            .with_context(|| format!("read task file {path}"))?;
        let j: serde_json::Value = serde_json::from_str(&raw)
            .with_context(|| format!("parse task JSON {path}"))?;
        let get = |k: &str| {
            j.get(k)
                .and_then(|v| v.as_str())
                .map(str::to_string)
                .unwrap_or_default()
        };
        return Ok(Task {
            slug: get("slug"),
            instructions: get("instructions"),
            starter_code: get("starter_code"),
            test_code: get("test_code"),
            cargo_toml: get("cargo_toml"),
        });
    }

    if let Some(slug) = &args.problem_slug {
        let problems = x402_soul::opus_bench::load_embedded_problems();
        let p = problems
            .into_iter()
            .find(|p| &p.slug == slug)
            .ok_or_else(|| anyhow!("no embedded problem with slug {slug}"))?;
        return Ok(p.into());
    }

    Err(anyhow!("pass --task-file or --problem-slug"))
}

/// Drop solution into a fresh Cargo crate and run `cargo test`. Returns
/// `Ok(())` on pass and `Err(message)` on fail (with stdout+stderr attached).
fn run_cargo_test(workdir: &Path, task: &Task, solution: &str) -> Result<(), String> {
    // Write lib.rs
    let src = workdir.join("src");
    fs::create_dir_all(&src).map_err(|e| format!("mkdir src: {e}"))?;
    fs::write(src.join("lib.rs"), solution).map_err(|e| format!("write lib.rs: {e}"))?;

    // Write tests
    let tests = workdir.join("tests");
    fs::create_dir_all(&tests).map_err(|e| format!("mkdir tests: {e}"))?;
    let test_file = format!("{}.rs", task.slug.replace('-', "_"));
    fs::write(tests.join(&test_file), &task.test_code)
        .map_err(|e| format!("write test file: {e}"))?;

    // Write Cargo.toml if one wasn't specified, use a minimal default.
    let cargo_toml = if task.cargo_toml.trim().is_empty() {
        format!(
            "[package]\nname = \"{slug}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\npath = \"src/lib.rs\"\n",
            slug = task.slug.replace('-', "_"),
        )
    } else {
        task.cargo_toml.clone()
    };
    fs::write(workdir.join("Cargo.toml"), cargo_toml)
        .map_err(|e| format!("write Cargo.toml: {e}"))?;

    // Run `cargo test`.
    let out = Command::new("cargo")
        .args(["test", "--quiet", "--color=never"])
        .current_dir(workdir)
        .output()
        .map_err(|e| format!("spawn cargo: {e}"))?;

    let mut msg = String::new();
    if !out.stdout.is_empty() {
        msg.push_str(&String::from_utf8_lossy(&out.stdout));
    }
    if !out.stderr.is_empty() {
        msg.push_str(&String::from_utf8_lossy(&out.stderr));
    }

    if out.status.success() {
        Ok(())
    } else {
        // Truncate to 4 kB — model doesn't need more than this.
        if msg.len() > 4096 {
            let head = &msg[..2048];
            let tail = &msg[msg.len() - 2048..];
            msg = format!("{head}\n... [truncated] ...\n{tail}");
        }
        Err(msg)
    }
}

fn build_initial_problem(task: &Task) -> BenchmarkProblem {
    BenchmarkProblem {
        slug: task.slug.clone(),
        difficulty: "tier1".to_string(),
        instructions: task.instructions.clone(),
        starter_code: task.starter_code.clone(),
        test_code: task.test_code.clone(),
        cargo_toml: task.cargo_toml.clone(),
    }
}

fn build_correction_problem(task: &Task, previous: &str, error: &str) -> BenchmarkProblem {
    // Re-uses the same prompt template as the benchmark generator, but wraps
    // the previous attempt + error into the `instructions` field so the model
    // sees them as part of the natural task description.
    let instructions = format!(
        "{orig}\n\n\
         ## Your previous attempt\n```rust\n{prev}\n```\n\n\
         ## Cargo test failed with this output\n```\n{err}\n```\n\n\
         Produce a corrected implementation. Keep what already works; fix only what the errors require.",
        orig = task.instructions,
        prev = previous,
        err = error,
    );
    BenchmarkProblem {
        slug: task.slug.clone(),
        difficulty: "tier1".to_string(),
        instructions,
        starter_code: task.starter_code.clone(),
        test_code: task.test_code.clone(),
        cargo_toml: task.cargo_toml.clone(),
    }
}

fn scratch_dir(slug: &str) -> PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!("rust-agent-{slug}-{}", std::process::id()));
    p
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rust_agent=info,tempo_x402_paper=info".parse().unwrap()),
        )
        .init();

    let args = Args::parse();
    let task = load_task(&args)?;
    tracing::info!(slug = %task.slug, "rust-agent: loaded task");

    let generator = CandleGenerator::from_dir(
        "rust-agent".to_string(),
        &args.model_dir,
        args.temperature as f64,
    )?
    .with_max_tokens(args.max_tokens);

    let workdir = scratch_dir(&task.slug);
    fs::create_dir_all(&workdir)
        .with_context(|| format!("create workdir {:?}", workdir))?;

    let mut last_solution = String::new();
    let mut last_error = String::new();
    let mut passed = false;

    for round in 1..=args.max_rounds {
        tracing::info!(round, "rust-agent: generating");
        let problem = if round == 1 {
            build_initial_problem(&task)
        } else {
            build_correction_problem(&task, &last_solution, &last_error)
        };

        let solution = generator
            .generate(&problem)
            .await
            .map_err(|e| anyhow!("generate: {e}"))?;

        last_solution = solution.clone();
        match run_cargo_test(&workdir, &task, &solution) {
            Ok(()) => {
                tracing::info!(round, "rust-agent: PASSED");
                passed = true;
                break;
            }
            Err(err) => {
                tracing::warn!(round, "rust-agent: FAILED; will retry");
                last_error = err;
            }
        }
    }

    // Emit final result
    if let Some(path) = &args.output {
        fs::write(path, &last_solution)
            .with_context(|| format!("write output {path}"))?;
        tracing::info!(output = %path, "rust-agent: solution saved");
    }

    if !args.keep_workdir {
        let _ = fs::remove_dir_all(&workdir);
    } else {
        tracing::info!(workdir = ?workdir, "rust-agent: keeping workdir");
    }

    if passed {
        println!("PASS");
        println!("---\n{last_solution}");
        Ok(())
    } else {
        println!("FAIL");
        println!("---\n{last_solution}\n---\n{last_error}");
        std::process::exit(1);
    }
}

