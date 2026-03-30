//! Local code generation model — Phase 3.
//!
//! 350M param Rust-specialist transformer. Trained from benchmark
//! solutions + commit diffs. Tries local generation first, falls
//! back to Gemini if confidence is low.
//!
//! NOT YET IMPLEMENTED — this module defines the target architecture
//! and training data pipeline interface. The colony watches Ψ(t) and
//! `ready_for_phase3()` to decide when to start building this.

/// Target architecture constants (Phase 3).
pub const CODEGEN_D_MODEL: usize = 768;
pub const CODEGEN_N_HEADS: usize = 12;
pub const CODEGEN_N_LAYERS: usize = 12;
pub const CODEGEN_D_FF: usize = 3072;
pub const CODEGEN_VOCAB_SIZE: usize = 8192; // BPE tokenizer
pub const CODEGEN_MAX_SEQ: usize = 1024;
pub const CODEGEN_PARAMS: usize = 350_000_000;

/// Training data source types.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum TrainingSource {
    /// Verified benchmark solution (passed tests).
    BenchmarkSolution {
        problem_id: String,
        code: String,
        passed: bool,
        tier: u32,
    },
    /// Commit diff with quality score from the quality model.
    CommitDiff {
        sha: String,
        diff: String,
        quality_score: f32,
    },
    /// Solution imported from a colony peer.
    PeerSolution {
        peer_id: String,
        problem_id: String,
        code: String,
    },
}

/// Readiness check: should we start building the local model?
///
/// Conditions:
/// - Ψ(t) > 0.5 (colony is healthy and learning)
/// - >500 training examples accumulated
/// - benchmark pass@1 > 60% (baseline competence established)
///
/// The colony watches these signals and begins Phase 3 when ready.
pub fn ready_for_phase3(psi: f64, training_examples: usize, pass_at_1: f64) -> bool {
    psi > 0.5 && training_examples > 500 && pass_at_1 > 60.0
}

/// Estimated memory usage for the target model at fp16.
pub const CODEGEN_MEMORY_MB: usize = CODEGEN_PARAMS * 2 / (1024 * 1024); // ~700 MB
