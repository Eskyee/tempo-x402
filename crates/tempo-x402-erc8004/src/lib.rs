//! ERC-8004 (Trustless Agents) integration for the Tempo blockchain.
//!
//! Provides client functions for three on-chain registries:
//! - **Identity** ([`identity`]) — agent NFT minting, metadata, recovery
//! - **Reputation** ([`reputation`]) — feedback submission and queries
//! - **Validation** ([`validation`]) — pluggable validator contracts (future)
//!
//! Contract addresses are read from environment variables, defaulting to
//! `Address::ZERO` (no-op) until contracts are deployed on Tempo Moderato.

use alloy::primitives::Address;

pub mod contracts;
pub mod identity;
pub mod recovery;
pub mod reputation;
pub mod types;
pub mod validation;

// Re-exports
pub use types::{AgentId, AgentMetadata, ReputationScore};

// ── Contract address configuration ──────────────────────────────────────

/// Get the identity registry contract address.
///
/// Reads from `ERC8004_IDENTITY_REGISTRY` env var, defaults to `Address::ZERO`.
pub fn identity_registry() -> Address {
    std::env::var("ERC8004_IDENTITY_REGISTRY")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(Address::ZERO)
}

/// Get the reputation registry contract address.
///
/// Reads from `ERC8004_REPUTATION_REGISTRY` env var, defaults to `Address::ZERO`.
pub fn reputation_registry() -> Address {
    std::env::var("ERC8004_REPUTATION_REGISTRY")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(Address::ZERO)
}

/// Get the validation registry contract address.
///
/// Reads from `ERC8004_VALIDATION_REGISTRY` env var, defaults to `Address::ZERO`.
pub fn validation_registry() -> Address {
    std::env::var("ERC8004_VALIDATION_REGISTRY")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(Address::ZERO)
}

/// Check whether ERC-8004 identity minting is enabled.
///
/// Reads `ERC8004_AUTO_MINT` env var (default: false).
pub fn auto_mint_enabled() -> bool {
    std::env::var("ERC8004_AUTO_MINT")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false)
}

/// Check whether reputation submission is enabled.
///
/// Reads `ERC8004_REPUTATION_ENABLED` env var (default: false).
pub fn reputation_enabled() -> bool {
    std::env::var("ERC8004_REPUTATION_ENABLED")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false)
}

/// Get the configured recovery address (if any).
///
/// Reads `ERC8004_RECOVERY_ADDRESS` env var.
pub fn recovery_address() -> Option<Address> {
    std::env::var("ERC8004_RECOVERY_ADDRESS")
        .ok()
        .and_then(|s| s.parse().ok())
}
