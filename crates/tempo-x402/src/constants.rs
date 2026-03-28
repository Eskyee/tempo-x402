//! Chain configuration and well-known constants for the Tempo blockchain.
//!
//! Contains the Tempo Moderato chain ID, default token address (pathUSD),
//! RPC endpoint, and a runtime [`ChainConfig`] struct for multi-chain support.

use alloy::primitives::Address;

/// Tempo Moderato chain ID.
pub const TEMPO_CHAIN_ID: u64 = 42431;

/// CAIP-2 network identifier for Tempo Moderato.
pub const TEMPO_NETWORK: &str = "eip155:42431";

/// x402 scheme name for TIP-20 payments on Tempo.
pub const SCHEME_NAME: &str = "tempo-tip20";

/// pathUSD token address on Tempo Moderato testnet.
pub const DEFAULT_TOKEN: Address = Address::new([
    0x20, 0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00,
]);

/// pathUSD has 6 decimal places.
pub const TOKEN_DECIMALS: u32 = 6;

/// Default RPC endpoint for Tempo Moderato.
pub const RPC_URL: &str = "https://rpc.moderato.tempo.xyz";

/// Block explorer base URL.
pub const EXPLORER_BASE: &str = "https://explore.moderato.tempo.xyz";

/// Read the token address from TEMPO_TOKEN env var, falling back to DEFAULT_TOKEN.
pub fn env_token() -> Address {
    std::env::var("TEMPO_TOKEN")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_TOKEN)
}

/// Read the CAIP-2 network string from CHAIN_ID env var, falling back to testnet.
pub fn env_network() -> String {
    let chain_id: u64 = std::env::var("CHAIN_ID")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(TEMPO_CHAIN_ID);
    if chain_id == TEMPO_CHAIN_ID {
        TEMPO_NETWORK.to_string()
    } else {
        format!("eip155:{chain_id}")
    }
}

/// Runtime chain configuration. Decouples scheme implementations from
/// compile-time constants, enabling multi-chain support.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChainConfig {
    pub chain_id: u64,
    pub network: String,
    pub scheme_name: String,
    pub default_token: Address,
    pub token_decimals: u32,
    pub rpc_url: String,
    pub explorer_base: String,
    pub eip712_domain_name: String,
    pub eip712_domain_version: String,
}

impl Default for ChainConfig {
    /// Defaults to Tempo Moderato configuration.
    /// Reads from env vars if set: TEMPO_TOKEN, CHAIN_ID, RPC_URL.
    fn default() -> Self {
        let chain_id: u64 = std::env::var("CHAIN_ID")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(TEMPO_CHAIN_ID);

        let network = if chain_id == TEMPO_CHAIN_ID {
            TEMPO_NETWORK.to_string()
        } else {
            format!("eip155:{chain_id}")
        };

        let default_token = std::env::var("TEMPO_TOKEN")
            .ok()
            .and_then(|s| s.parse::<Address>().ok())
            .unwrap_or(DEFAULT_TOKEN);

        let rpc_url = std::env::var("RPC_URL")
            .unwrap_or_else(|_| RPC_URL.to_string());

        Self {
            chain_id,
            network,
            scheme_name: SCHEME_NAME.to_string(),
            default_token,
            token_decimals: TOKEN_DECIMALS,
            rpc_url,
            explorer_base: EXPLORER_BASE.to_string(),
            eip712_domain_name: "x402-tempo".to_string(),
            eip712_domain_version: "1".to_string(),
        }
    }
}
