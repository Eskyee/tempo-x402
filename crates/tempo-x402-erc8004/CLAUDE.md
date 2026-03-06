# tempo-x402-erc8004

Library crate. ERC-8004 (Trustless Agents) integration — on-chain agent identity (ERC-721 NFT), reputation, and validation registries.

One crate, three registry modules. Contracts share alloy, chain config, and deployment lifecycle.

## Depends On

- `x402` (core: X402Error, ChainConfig)
- `x402-wallet` (sign_message, recover_message_signer for recovery proofs)

## Structure

- `contracts.rs` — sol! bindings for IAgentIdentity, IAgentReputation, IAgentValidation
- `identity.rs` — mint, owner_of, set_recovery_address, recover_agent, update_metadata, get_metadata_uri
- `reputation.rs` — submit_feedback, get_reputation
- `validation.rs` — register_validator, remove_validator, execute_with_validation
- `types.rs` — AgentId, ReputationScore, AgentMetadata
- `recovery.rs` — Recovery proof construction + verification using x402-wallet
- `deploy.rs` — Self-deployment of all 3 contracts from embedded compiled bytecode

## Non-Obvious Patterns

- Contract addresses from env vars with `Address::ZERO` defaults (not deployed yet)
- Follows `tip20.rs` pattern: timeout-wrapped send + receipt, revert checks
- Recovery proofs use EIP-191 sign_message from x402-wallet (WASM-safe crypto)
- All chain calls are async + tokio timeout-guarded
- Self-deploy: node deploys its own contracts when `ERC8004_AUTO_MINT=true` and no registry addresses configured
- Deployed addresses persisted to `/data/erc8004_registries.json` (configurable via `ERC8004_REGISTRIES_PATH`)

## If You're Changing...

- **Contract ABI**: Update sol! macros in `contracts.rs`
- **Contract addresses**: Update env var names in `lib.rs`
- **Used by**: `x402-node` (identity minting, reputation submission), `x402-soul` (reputation tools)
