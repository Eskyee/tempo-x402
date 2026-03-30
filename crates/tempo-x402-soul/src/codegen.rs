//! Code generation model orchestration — Phase 3.
//!
//! Thin wrapper around the model crate's BPE tokenizer and (future)
//! code generation transformer. Handles:
//! - Loading/saving BPE tokenizer from soul_state
//! - Periodic training on accumulated benchmark solutions
//! - (Future) Local-first code generation with Gemini fallback

use crate::db::SoulDatabase;

/// Load the BPE tokenizer from soul_state.
pub fn load_tokenizer(db: &SoulDatabase) -> x402_model::bpe::BpeTokenizer {
    match db.get_state("codegen_bpe_tokenizer").ok().flatten() {
        Some(json) if !json.is_empty() => {
            x402_model::bpe::BpeTokenizer::from_json(&json)
                .unwrap_or_else(|| x402_model::bpe::BpeTokenizer::new(8192))
        }
        _ => x402_model::bpe::BpeTokenizer::new(8192),
    }
}

/// Save the BPE tokenizer to soul_state.
pub fn save_tokenizer(db: &SoulDatabase, tok: &x402_model::bpe::BpeTokenizer) {
    let json = tok.to_json();
    if let Err(e) = db.set_state("codegen_bpe_tokenizer", &json) {
        tracing::warn!(error = %e, "Failed to save BPE tokenizer");
    }
}

/// Train the BPE tokenizer on accumulated benchmark solutions.
///
/// Called periodically (every ~50 cycles). Loads all stored solutions,
/// concatenates their code, and trains the tokenizer to learn Rust
/// source code patterns. More solutions → better tokenization → better
/// code generation when the full model is built.
pub fn train_tokenizer(db: &SoulDatabase) {
    // Load accumulated solutions
    let solutions: Vec<serde_json::Value> = db
        .get_state("codegen_solutions")
        .ok()
        .flatten()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();

    if solutions.is_empty() {
        return;
    }

    // Concatenate all solution code into training corpus
    let mut corpus = String::new();
    for sol in &solutions {
        if let Some(code) = sol.get("code").and_then(|v| v.as_str()) {
            corpus.push_str(code);
            corpus.push('\n');
        }
    }

    if corpus.len() < 100 {
        return; // Not enough data
    }

    // Train tokenizer
    let mut tok = load_tokenizer(db);
    let before_vocab = tok.current_vocab_size();
    tok.train(&corpus);
    let after_vocab = tok.current_vocab_size();
    let ratio = tok.compression_ratio(&corpus);

    save_tokenizer(db, &tok);

    tracing::info!(
        solutions = solutions.len(),
        corpus_bytes = corpus.len(),
        vocab_before = before_vocab,
        vocab_after = after_vocab,
        compression_ratio = format!("{ratio:.2}"),
        "BPE tokenizer trained on benchmark solutions"
    );
}

/// Load the code gen model from soul_state.
pub fn load_model(db: &SoulDatabase) -> x402_model::codegen::CodeGenModel {
    match db.get_state("codegen_model").ok().flatten() {
        Some(json) if json.len() > 100 => {
            x402_model::codegen::CodeGenModel::from_json(&json)
                .unwrap_or_default()
        }
        _ => x402_model::codegen::CodeGenModel::new(),
    }
}

/// Save the code gen model to soul_state.
pub fn save_model(db: &SoulDatabase, model: &x402_model::codegen::CodeGenModel) {
    let json = model.to_json();
    if let Err(e) = db.set_state("codegen_model", &json) {
        tracing::warn!(error = %e, "Failed to save codegen model");
    }
}

/// Train the code generation model on accumulated solutions.
///
/// Loads solutions, tokenizes with BPE, feeds token sequences to the model.
/// Called periodically alongside BPE training.
pub fn train_model(db: &SoulDatabase) {
    let tok = load_tokenizer(db);
    if tok.merges.is_empty() {
        return; // BPE not trained yet
    }

    let solutions: Vec<serde_json::Value> = db
        .get_state("codegen_solutions")
        .ok()
        .flatten()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();

    if solutions.len() < 5 {
        return; // Not enough data
    }

    let mut model = load_model(db);
    let mut total_loss = 0.0f32;
    let mut trained = 0u32;

    // Train on up to 20 recent solutions per cycle
    for sol in solutions.iter().rev().take(20) {
        let Some(code) = sol.get("code").and_then(|v| v.as_str()) else {
            continue;
        };

        // Tokenize with BPE
        let mut tokens = vec![x402_model::bpe::BOS_TOKEN];
        tokens.extend(tok.encode(code));
        tokens.push(x402_model::bpe::EOS_TOKEN);

        // Truncate to max seq length
        if tokens.len() > x402_model::codegen::SMALL_MAX_SEQ {
            tokens.truncate(x402_model::codegen::SMALL_MAX_SEQ);
        }

        if tokens.len() < 3 {
            continue;
        }

        // Train on sliding windows
        let window_size = 64.min(tokens.len());
        for start in (0..tokens.len().saturating_sub(window_size)).step_by(32) {
            let end = (start + window_size).min(tokens.len());
            let window = &tokens[start..end];
            let loss = model.train_step(window, 0.001);
            total_loss += loss;
            trained += 1;
        }
    }

    if trained > 0 {
        save_model(db, &model);
        tracing::info!(
            trained,
            loss = format!("{:.4}", total_loss / trained as f32),
            running_loss = format!("{:.4}", model.running_loss),
            steps = model.train_steps,
            params = model.param_count(),
            "Code gen model training cycle"
        );
    }
}

/// Generate code given a prompt. Returns None if model not ready.
///
/// Uses greedy decoding (argmax) for now. Temperature sampling later.
pub fn generate(db: &SoulDatabase, prompt: &str, max_tokens: usize) -> Option<String> {
    let tok = load_tokenizer(db);
    if tok.merges.is_empty() {
        return None;
    }

    let model = load_model(db);
    if model.train_steps < 100 {
        return None; // Not enough training
    }

    // Tokenize prompt
    let mut tokens = vec![x402_model::bpe::BOS_TOKEN];
    tokens.extend(tok.encode(prompt));

    // Generate token by token (greedy)
    for _ in 0..max_tokens {
        if tokens.len() >= model.max_seq {
            break;
        }

        let logits = model.forward(&tokens);

        // Argmax
        let next_token = logits
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(idx, _)| idx as u32)
            .unwrap_or(x402_model::bpe::EOS_TOKEN);

        if next_token == x402_model::bpe::EOS_TOKEN {
            break;
        }

        tokens.push(next_token);
    }

    // Decode (skip BOS + prompt tokens)
    let prompt_len = 1 + tok.encode(prompt).len();
    if tokens.len() <= prompt_len {
        return None;
    }

    let generated = tok.decode(&tokens[prompt_len..]);
    if generated.trim().is_empty() {
        return None;
    }

    Some(generated)
}

/// Get status for observability.
pub fn status(db: &SoulDatabase) -> serde_json::Value {
    let tok = load_tokenizer(db);
    let training_count: u64 = db
        .get_state("codegen_training_count")
        .ok()
        .flatten()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let solutions_count: usize = db
        .get_state("codegen_solutions")
        .ok()
        .flatten()
        .and_then(|s| serde_json::from_str::<Vec<serde_json::Value>>(&s).ok())
        .map(|v| v.len())
        .unwrap_or(0);

    let phase3_ready = x402_model::codegen::ready_for_phase3(
        db.get_state("psi_value")
            .ok()
            .flatten()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0),
        solutions_count,
        db.get_state("benchmark_pass_at_1")
            .ok()
            .flatten()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0),
    );

    let model = load_model(db);

    serde_json::json!({
        "bpe_vocab_size": tok.current_vocab_size(),
        "bpe_merges": tok.merges.len(),
        "training_count": training_count,
        "solutions_stored": solutions_count,
        "phase3_ready": phase3_ready,
        "model_params": model.param_count(),
        "model_steps": model.train_steps,
        "model_loss": format!("{:.4}", model.running_loss),
        "target_params": x402_model::codegen::CODEGEN_PARAMS,
        "target_vocab": x402_model::codegen::CODEGEN_VOCAB_SIZE,
    })
}
