//! GPU inference backend using `candle`.
//!
//! Loads a Hugging Face-style model directory containing:
//!   - model.safetensors (or model-*.safetensors shards)
//!   - config.json
//!   - tokenizer.json
//!
//! Runs on CUDA when the `cuda` feature is enabled (default), falls back to CPU.
//! No external HTTP server, no GGUF quantization step — the same safetensors we
//! trained with are the ones we benchmark.
//!
//! # Example
//!
//! ```no_run
//! # use tempo_x402_paper::backends::candle::CandleGenerator;
//! let gen = CandleGenerator::from_dir(
//!     "qwen-0.5b-full-ft",
//!     "selfplay_runs_v2/checkpoints/full_ft/merged",
//!     0.0,
//! ).unwrap();
//! ```

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{anyhow, Context, Result};
use candle_core::{DType, Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::generation::LogitsProcessor;
use candle_transformers::models::qwen2::{Config as Qwen2Config, ModelForCausalLM};
use tokenizers::Tokenizer;
use tokio::sync::Mutex;

use crate::runner::CodeGenerator;
use x402_soul::benchmark::BenchmarkProblem;

/// System prompt — identical wording to the HTTP `LocalModelGenerator` so
/// candle numbers are directly comparable to prior llama-server runs.
const SYSTEM_PROMPT: &str = "You are an expert Rust programmer. Output ONLY complete Rust code for src/lib.rs. No explanations, no markdown.";

/// Qwen2 chat-template special tokens.
const IM_START: &str = "<|im_start|>";
const IM_END: &str = "<|im_end|>";

pub struct CandleGenerator {
    name: String,
    inner: Arc<Mutex<Inner>>,
    temperature: f64,
    max_new_tokens: usize,
    top_p: f64,
    seed: u64,
}

/// The `&mut`-heavy parts live behind a Mutex because the
/// `CodeGenerator::generate` method takes `&self` but `ModelForCausalLM::forward`
/// mutates internal KV cache state.
struct Inner {
    model: ModelForCausalLM,
    tokenizer: Tokenizer,
    device: Device,
    im_end_id: u32,
    eos_id: u32,
}

impl CandleGenerator {
    /// Load from a local directory laid out like an HF model snapshot.
    pub fn from_dir<P: AsRef<Path>>(
        name: impl Into<String>,
        dir: P,
        temperature: f64,
    ) -> Result<Self> {
        let dir = dir.as_ref();

        let device = pick_device()?;
        tracing::info!(device = %device_label(&device), "candle: device selected");

        let tokenizer_path = dir.join("tokenizer.json");
        let tokenizer = Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| anyhow!("load tokenizer {:?}: {e}", tokenizer_path))?;

        let config_path = dir.join("config.json");
        let config_raw = std::fs::read_to_string(&config_path)
            .with_context(|| format!("read {:?}", config_path))?;
        let config: Qwen2Config = serde_json::from_str(&config_raw)
            .with_context(|| format!("parse Qwen2 config at {:?}", config_path))?;

        let weight_paths = collect_safetensors(dir)?;
        if weight_paths.is_empty() {
            return Err(anyhow!("no *.safetensors files found in {:?}", dir));
        }
        tracing::info!(shards = weight_paths.len(), "candle: loading weights");

        // BF16 is the accepted sweet spot on Ampere+ (our 3090). F16 also works.
        let dtype = if device.is_cuda() { DType::BF16 } else { DType::F32 };
        let vb = unsafe {
            VarBuilder::from_mmaped_safetensors(&weight_paths, dtype, &device)?
        };
        let model = ModelForCausalLM::new(&config, vb)?;

        let im_end_id = tokenizer
            .token_to_id(IM_END)
            .ok_or_else(|| anyhow!("tokenizer has no {} token", IM_END))?;
        // Qwen2 uses <|endoftext|> as EOS for non-chat completions.
        let eos_id = tokenizer.token_to_id("<|endoftext|>").unwrap_or(im_end_id);

        Ok(Self {
            name: name.into(),
            inner: Arc::new(Mutex::new(Inner {
                model,
                tokenizer,
                device,
                im_end_id,
                eos_id,
            })),
            temperature,
            max_new_tokens: 1024,
            top_p: 0.9,
            seed: 42,
        })
    }

    pub fn with_max_tokens(mut self, n: usize) -> Self {
        self.max_new_tokens = n;
        self
    }

    pub fn with_top_p(mut self, top_p: f64) -> Self {
        self.top_p = top_p;
        self
    }

    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    fn build_prompt(problem: &BenchmarkProblem) -> String {
        // Wrapping the user turn in the same template llama-server applied for Qwen2.
        let user = format!(
            "Write a complete Rust library (src/lib.rs) that passes these tests.\n\n\
             ## Problem: {slug}\n\n\
             ## Instructions\n{instr}\n\n\
             ## Tests\n```rust\n{tests}\n```\n\n\
             ## Starter Code\n```rust\n{starter}\n```\n\n\
             Output ONLY the Rust code for src/lib.rs. No explanations.",
            slug = problem.slug,
            instr = problem.instructions,
            tests = problem.test_code,
            starter = problem.starter_code,
        );
        format!(
            "{IM_START}system\n{SYSTEM_PROMPT}{IM_END}\n\
             {IM_START}user\n{user}{IM_END}\n\
             {IM_START}assistant\n"
        )
    }
}

impl Inner {
    fn generate(
        &mut self,
        prompt: &str,
        temperature: f64,
        top_p: f64,
        seed: u64,
        max_new_tokens: usize,
    ) -> Result<String> {
        // Reset KV cache so each problem is independent.
        self.model.clear_kv_cache();

        let encoding = self
            .tokenizer
            .encode(prompt, true)
            .map_err(|e| anyhow!("tokenize: {e}"))?;
        let prompt_ids: Vec<u32> = encoding.get_ids().to_vec();
        if prompt_ids.is_empty() {
            return Err(anyhow!("empty prompt after tokenization"));
        }

        let temp_opt = if temperature <= 0.0 {
            None
        } else {
            Some(temperature)
        };
        let mut logits_processor =
            LogitsProcessor::new(seed, temp_opt, temp_opt.map(|_| top_p));

        let mut all_ids = prompt_ids.clone();
        let mut output_ids: Vec<u32> = Vec::with_capacity(max_new_tokens);

        for step in 0..max_new_tokens {
            let (context, offset) = if step == 0 {
                (all_ids.as_slice(), 0usize)
            } else {
                (&all_ids[all_ids.len() - 1..], all_ids.len() - 1)
            };
            let input = Tensor::new(context, &self.device)?.unsqueeze(0)?;
            let logits = self.model.forward(&input, offset)?;
            // logits: [1, seq, vocab] — take last position.
            let last = logits
                .squeeze(0)?
                .get(logits.dim(1)? - 1)?
                .to_dtype(DType::F32)?;
            let next = logits_processor.sample(&last)?;

            if next == self.im_end_id || next == self.eos_id {
                break;
            }
            all_ids.push(next);
            output_ids.push(next);
        }

        let text = self
            .tokenizer
            .decode(&output_ids, true)
            .map_err(|e| anyhow!("decode: {e}"))?;
        Ok(strip_code_fences(&text))
    }
}

#[async_trait::async_trait]
impl CodeGenerator for CandleGenerator {
    async fn generate(&self, problem: &BenchmarkProblem) -> Result<String, String> {
        let prompt = Self::build_prompt(problem);
        let temperature = self.temperature;
        let top_p = self.top_p;
        let seed = self.seed;
        let max_new = self.max_new_tokens;
        let inner = Arc::clone(&self.inner);

        // candle forward is synchronous + CPU/GPU-bound; run on blocking pool.
        tokio::task::spawn_blocking(move || {
            let mut guard = inner.blocking_lock();
            guard.generate(&prompt, temperature, top_p, seed, max_new)
        })
        .await
        .map_err(|e| format!("join error: {e}"))?
        .map_err(|e| format!("{e:#}"))
    }

    fn name(&self) -> &str {
        &self.name
    }
}

fn pick_device() -> Result<Device> {
    #[cfg(feature = "cuda")]
    {
        match Device::new_cuda(0) {
            Ok(d) => return Ok(d),
            Err(e) => tracing::warn!("candle: CUDA unavailable ({e}); falling back to CPU"),
        }
    }
    Ok(Device::Cpu)
}

fn device_label(d: &Device) -> &'static str {
    if d.is_cuda() {
        "cuda"
    } else if d.is_metal() {
        "metal"
    } else {
        "cpu"
    }
}

fn collect_safetensors(dir: &Path) -> Result<Vec<PathBuf>> {
    // Prefer sharded `model-*.safetensors` if present (matches HF naming).
    let mut shards: Vec<PathBuf> = std::fs::read_dir(dir)
        .with_context(|| format!("read_dir {:?}", dir))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| {
                    n.starts_with("model-") && n.ends_with(".safetensors")
                })
                .unwrap_or(false)
        })
        .collect();
    shards.sort();
    if !shards.is_empty() {
        return Ok(shards);
    }

    let single = dir.join("model.safetensors");
    if single.exists() {
        return Ok(vec![single]);
    }
    Ok(Vec::new())
}

/// Extract Rust code from markdown fences (same logic as the HTTP backend).
fn strip_code_fences(text: &str) -> String {
    let mut blocks = Vec::new();
    let mut search = text;
    while let Some(start) = search.find("```rust") {
        let after = &search[start + 7..];
        let code_start = if after.starts_with('\n') { 1 } else { 0 };
        if let Some(end) = after[code_start..].find("```") {
            blocks.push(after[code_start..code_start + end].trim().to_string());
            search = &after[code_start + end + 3..];
        } else {
            blocks.push(after[code_start..].trim().to_string());
            break;
        }
    }
    if blocks.is_empty() {
        let mut s = text;
        while let Some(start) = s.find("```") {
            let after = &s[start + 3..];
            let code_start = after.find('\n').map(|n| n + 1).unwrap_or(0);
            if let Some(end) = after[code_start..].find("```") {
                blocks.push(after[code_start..code_start + end].trim().to_string());
                s = &after[code_start + end + 3..];
            } else {
                blocks.push(after[code_start..].trim().to_string());
                break;
            }
        }
    }
    if let Some(best) = blocks.into_iter().max_by_key(|b| b.len()) {
        if !best.is_empty() {
            return best;
        }
    }
    text.trim().to_string()
}
