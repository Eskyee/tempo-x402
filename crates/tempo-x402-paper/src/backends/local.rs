//! Local model backend — runs GGUF models via llama-cpp for inference.
//!
//! Supports any GGUF model: Qwen, DeepSeek, CodeGemma, StarCoder, etc.
//! Downloads from HuggingFace if not present locally.
//!
//! NOTE: Requires `llama-cpp` feature to be enabled. When disabled,
//! this module provides a stub that returns an error.

use crate::runner::CodeGenerator;
use x402_soul::benchmark::BenchmarkProblem;

/// Local GGUF model for code generation.
///
/// Two modes:
/// - **Server mode** (recommended): Point to a running `llama-server` via `--server-url`.
///   Model stays loaded in memory — each problem is a fast HTTP call.
///   Start server: `llama-server -m model.gguf --port 8081 -c 4096`
/// - **CLI mode**: Spawns `llama-completion` per problem. Slow (reloads model each time).
pub struct LocalModelGenerator {
    name: String,
    /// GGUF model path (for CLI mode) or server URL (for server mode)
    model_path: String,
    /// If set, use HTTP API instead of subprocess
    server_url: Option<String>,
    n_ctx: u32,
    max_tokens: u32,
    temperature: f32,
    client: reqwest::Client,
}

impl LocalModelGenerator {
    pub fn new(name: String, model_path: String) -> Self {
        Self {
            name,
            model_path,
            server_url: None,
            n_ctx: 4096,
            max_tokens: 2048,
            temperature: 0.2,
            client: reqwest::Client::new(),
        }
    }

    /// Use a running llama-server instead of spawning processes.
    /// Much faster — model stays loaded in memory.
    pub fn with_server(mut self, url: String) -> Self {
        self.server_url = Some(url);
        self
    }

    pub fn with_ctx(mut self, n_ctx: u32) -> Self {
        self.n_ctx = n_ctx;
        self
    }

    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature;
        self
    }

    fn build_prompt(problem: &BenchmarkProblem) -> String {
        // Chat-style prompt that works with instruction-tuned models
        format!(
            "You are an expert Rust programmer. Write a complete Rust library (src/lib.rs) that passes these tests.\n\n\
             ## Problem: {}\n\n\
             ## Instructions\n{}\n\n\
             ## Tests\n```rust\n{}\n```\n\n\
             ## Starter Code\n```rust\n{}\n```\n\n\
             Output ONLY the Rust code for src/lib.rs. No explanations.",
            problem.slug,
            problem.instructions,
            problem.test_code,
            problem.starter_code,
        )
    }

    /// Run inference via llama-server HTTP API (fast — model stays loaded).
    async fn generate_via_server(&self, prompt: &str) -> Result<String, String> {
        let url = self.server_url.as_deref().unwrap_or("http://127.0.0.1:8081");
        let body = serde_json::json!({
            "prompt": prompt,
            "n_predict": self.max_tokens,
            "temperature": self.temperature,
            "stop": ["```\n", "\n\n\n\n"],
            "stream": false,
        });

        let resp = self
            .client
            .post(&format!("{url}/completion"))
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("server request failed (is llama-server running?): {e}"))?;

        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("server error: {text}"));
        }

        let json: serde_json::Value = resp.json().await.map_err(|e| format!("parse: {e}"))?;
        let content = json
            .get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let code = strip_code_fences(content);
        Ok(code.to_string())
    }

    /// Run inference using llama-completion (llama.cpp non-interactive binary).
    /// Avoids complex FFI — just shell out to the binary.
    async fn generate_via_cli(&self, prompt: &str) -> Result<String, String> {
        // Look for llama-completion in LLAMA_CPP_PATH or common locations
        let llama_bin = std::env::var("LLAMA_CPP_PATH").unwrap_or_else(|_| {
            // Check common build locations
            for path in &[
                "llama-completion",
                "/c/llama-cpp/build/bin/Release/llama-completion.exe",
                "/usr/local/bin/llama-completion",
                "./llama.cpp/build/bin/Release/llama-completion.exe",
                "./llama.cpp/build/bin/llama-completion",
            ] {
                if std::path::Path::new(path).exists() {
                    return path.to_string();
                }
            }
            "llama-completion".to_string()
        });

        let output = tokio::process::Command::new(&llama_bin)
            .args([
                "-m", &self.model_path,
                "--prompt", prompt,
                "-n", &self.max_tokens.to_string(),
                "--temp", &self.temperature.to_string(),
                "-c", &self.n_ctx.to_string(),
                "--no-display-prompt",
            ])
            .output()
            .await
            .map_err(|e| format!(
                "llama-completion not found (set LLAMA_CPP_PATH or build llama.cpp): {e}"
            ))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("llama-completion failed: {stderr}"));
        }

        let text = String::from_utf8_lossy(&output.stdout).to_string();
        // Strip ANSI escape codes and llama.cpp noise
        let cleaned: String = text
            .lines()
            .filter(|l| !l.starts_with("> ") && !l.contains("EOF by user"))
            .collect::<Vec<_>>()
            .join("\n");
        let code = strip_code_fences(&cleaned);
        Ok(code.to_string())
    }
}

#[async_trait::async_trait]
impl CodeGenerator for LocalModelGenerator {
    async fn generate(&self, problem: &BenchmarkProblem) -> Result<String, String> {
        let prompt = Self::build_prompt(problem);

        if self.server_url.is_some() {
            return self.generate_via_server(&prompt).await;
        }

        // CLI mode — check model file exists
        if !std::path::Path::new(&self.model_path).exists() {
            return Err(format!(
                "Model file not found: {}. Download a GGUF model from HuggingFace:\n  \
                 huggingface-cli download Qwen/Qwen2.5-Coder-0.5B-Instruct-GGUF \
                 qwen2.5-coder-0.5b-instruct-q4_k_m.gguf --local-dir models/",
                self.model_path
            ));
        }

        self.generate_via_cli(&prompt).await
    }

    fn name(&self) -> &str {
        &self.name
    }
}

fn strip_code_fences(text: &str) -> &str {
    let trimmed = text.trim();
    if let Some(after) = trimmed.strip_prefix("```rust") {
        if let Some(code) = after.strip_suffix("```") {
            return code.trim();
        }
        return after.trim();
    }
    if let Some(after) = trimmed.strip_prefix("```") {
        if let Some(code) = after.strip_suffix("```") {
            return code.trim();
        }
        return after.trim();
    }
    trimmed
}
