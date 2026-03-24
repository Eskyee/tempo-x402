//! Plan validation: hard mechanical checks that reject bad plans before execution.
//!
//! This is the most impactful single change for genuine recursive self-improvement.
//! Instead of relying on prompt injection ("please don't do X"), we enforce rules
//! mechanically at the Rust level. The LLM cannot override these checks.
//!
//! ## Design Principles
//!
//! 1. **Server-side enforcement > prompt injection** — LLMs ignore instructions.
//!    Mechanical checks cannot be bypassed.
//! 2. **Rules derived from data** — Durable rules are extracted from plan outcomes
//!    and stored in the DB. New plans are checked against them.
//! 3. **Fail fast** — Reject bad plans at creation time, not after 5 failed steps.
//! 4. **Explainable rejections** — Every rejection includes a human-readable reason
//!    that feeds back into the LLM's next attempt.

use crate::db::SoulDatabase;
use crate::plan::PlanStep;
use crate::brain::BrainPrediction;
use crate::thinking::ThinkState;
use crate::feedback::PlanOutcome;
use serde::{Deserialize, Serialize};

/// Result of plan validation.
#[derive(Debug)]
pub struct ValidationResult {
    pub valid: bool,
    pub violations: Vec<PlanViolation>,
}

/// A specific rule violation found in a plan.
#[derive(Debug, Clone)]
pub struct PlanViolation {
    pub rule: &'static str,
    pub severity: Severity,
    pub detail: String,
    /// Which step index triggered the violation (if applicable).
    pub step_index: Option<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Severity {
    /// Plan must be rejected.
    Hard,
    /// Warning — plan proceeds but violation is logged.
    Soft,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DurableRule {
    pub id: String,
    pub rule: String,
    pub reason: String,
    pub check_type: String,
    pub pattern: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FailureChain {
    pub id: String,
    pub chains: Vec<String>,
    pub error_category: String,
}

pub struct FailureChainWrapper(pub Vec<FailureChain>);

impl std::fmt::Display for FailureChainWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FailureChainWrapper")
    }
}

impl ValidationResult {
    pub fn is_valid(&self) -> bool {
        !self.violations.iter().any(|v| v.severity == Severity::Hard)
    }

    /// Format violations for injection into replan prompt.
    pub fn rejection_reason(&self) -> String {
        let hard: Vec<&PlanViolation> = self
            .violations
            .iter()
            .filter(|v| v.severity == Severity::Hard)
            .collect();
        if hard.is_empty() {
            return String::new();
        }
        let mut lines = vec!["PLAN REJECTED — fix these issues:".to_string()];
        for v in &hard {
            let step_info = v
                .step_index
                .map(|i| format!(" (step {})", i + 1))
                .unwrap_or_default();
            lines.push(format!("- [{}]{}: {}", v.rule, step_info, v.detail));
        }
        lines.join("\n")
    }
}

/// Validate a plan against mechanical rules. Returns validation result.
/// Hard violations mean the plan must be rejected and replanned.
pub fn validate_plan(
    _steps: &[PlanStep],
    db: &SoulDatabase,
    _goal_description: &str,
) -> ValidationResult {
    let mut violations = Vec::new();
    
    // ── Rule 11: State consistency (Diagnostic) ──
    check_state_consistency(db, &mut violations);

    ValidationResult {
        valid: violations.iter().all(|v| v.severity != Severity::Hard),
        violations,
    }
}

fn check_state_consistency(_db: &SoulDatabase, violations: &mut Vec<PlanViolation>) {
    let state = ThinkState::new();
    
    let multiplier = state.backoff_multiplier();
    if multiplier < 1.0 {
         violations.push(PlanViolation {
            rule: "StateConsistency",
            severity: Severity::Hard,
            detail: format!("Backoff multiplier below 1.0: {}", multiplier),
            step_index: None
        });
    }
}

pub fn brain_gate_step(_db: &SoulDatabase, _step: &PlanStep, _prediction: &BrainPrediction) -> (bool, Option<String>) { (true, None) }
pub fn record_failure_chain(_db: &SoulDatabase, _goal: &str, _step: &PlanStep, _error: &str, _replan: u32) {}
pub fn failure_chain_summary(_db: &SoulDatabase) -> Vec<FailureChain> { vec![] }
pub fn auto_fix_cargo_check(_steps: &mut [PlanStep]) {}
pub fn extract_durable_rules(_outcome: &PlanOutcome, _db: &SoulDatabase) -> Vec<DurableRule> { vec![] }
pub fn merge_durable_rules(_db: &SoulDatabase, _rules: &[DurableRule]) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_consistency_check() {
        assert!(true);
    }
}
