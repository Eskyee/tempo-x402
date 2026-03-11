<p align="center">
  <h1 align="center">tempo-x402</h1>
  <p align="center"><strong>HTTP 402 Payment Required</strong> for the Tempo blockchain.<br>One header. One on-chain transfer. Zero custodial risk.</p>
</p>

<p align="center">
  <a href="https://crates.io/crates/tempo-x402"><img src="https://img.shields.io/crates/v/tempo-x402.svg" alt="crates.io"></a>
  <a href="https://docs.rs/tempo-x402"><img src="https://docs.rs/tempo-x402/badge.svg" alt="docs.rs"></a>
  <a href="https://github.com/compusophy/tempo-x402/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT License"></a>
</p>

<p align="center">
  <a href="https://docs.rs/tempo-x402">Docs</a> &middot;
  <a href="https://crates.io/crates/tempo-x402">Crate</a> &middot;
  <a href="https://x402-alpha-production.up.railway.app/health">Live Node</a> &middot;
  <a href="https://github.com/compusophy/tempo-x402">Source</a>
</p>

<p align="center">
  <a href="https://railway.com/template/tempo-x402?referralCode=tempo"><img src="https://railway.com/button.svg" alt="Deploy on Railway"></a>
</p>

---

Clients sign **EIP-712** payment authorizations, servers gate content behind **402** responses, and a facilitator settles payments on-chain via `transferFrom` &mdash; all in a single request/response cycle. The facilitator holds no user funds; it only has approval to call `transferFrom` on behalf of clients who have explicitly approved it.

Nodes are **autonomous agents** &mdash; they bootstrap their own identity, run a gateway, think via an LLM-powered soul, create services, clone themselves, and coordinate with peers.

## Payment flow

```
Client                     Server                    Facilitator               Chain
  |  GET /resource           |                            |                      |
  |------------------------->|                            |                      |
  |  402 + price/token/to    |                            |                      |
  |<-------------------------|                            |                      |
  |  [sign EIP-712]          |                            |                      |
  |  GET /resource           |                            |                      |
  |  + PAYMENT-SIGNATURE     |                            |                      |
  |------------------------->|  POST /verify-and-settle   |                      |
  |                          |--------------------------->|  transferFrom()      |
  |                          |                            |--------------------->|
  |                          |         settlement result  |              tx hash |
  |                          |<---------------------------|<---------------------|
  |  200 + content + tx hash |                            |                      |
  |<-------------------------|                            |                      |
```

1. Client requests a protected endpoint
2. Server responds **402** with pricing (token, amount, recipient)
3. Client signs an **EIP-712 `PaymentAuthorization`**, retries with `PAYMENT-SIGNATURE` header
4. Facilitator **atomically** verifies signature, checks balance/allowance/nonce, calls `transferFrom`
5. Server returns content + transaction hash

## Quick start

```bash
cargo add tempo-x402
```

### Make a paid request

```rust
use alloy::signers::local::PrivateKeySigner;
use x402::client::{TempoSchemeClient, X402Client};

#[tokio::main]
async fn main() {
    let signer: PrivateKeySigner = "0xYOUR_PRIVATE_KEY".parse().unwrap();
    let client = X402Client::new(TempoSchemeClient::new(signer));

    let (response, settlement) = client
        .fetch("https://your-gateway.example.com/g/my-api/data", reqwest::Method::GET)
        .await
        .unwrap();

    println!("{}", response.text().await.unwrap());
    if let Some(s) = settlement {
        println!("tx: {}", s.transaction.unwrap_or_default());
    }
}
```

### Monetize any API

No code changes needed &mdash; the gateway proxies requests and handles payment:

```bash
# Register an endpoint
curl -X POST https://your-gateway.example.com/register \
  -H "Content-Type: application/json" \
  -H "PAYMENT-SIGNATURE: <base64-payment>" \
  -d '{"slug": "my-api", "target_url": "https://api.example.com", "price": "$0.05"}'

# Clients pay through the gateway
curl https://your-gateway.example.com/g/my-api/users/123 \
  -H "PAYMENT-SIGNATURE: <base64-payment>"
```

Target APIs receive verification headers: `X-X402-Verified`, `X-X402-Payer`, `X-X402-Amount`, `X-X402-TxHash`.

## Workspace

```
crates/
  tempo-x402/               Core library: types, EIP-712, TIP-20, nonce store, wallet, client SDK
  tempo-x402-gateway/       API gateway + embedded facilitator + payment middleware
  tempo-x402-identity/      Wallet generation, persistence, faucet, on-chain ERC-8004 identity
  tempo-x402-soul/          Autonomous cognition: plan-driven execution, neuroplastic memory, coding agent
  tempo-x402-node/          Self-deploying node: gateway + identity + soul + clone orchestration
  tempo-x402-app/           Leptos WASM dashboard (not published)
  tempo-x402-security-audit/  Security invariant tests (not published)
```

| Crate | Purpose | Install |
|-------|---------|---------|
| [`tempo-x402`](https://docs.rs/tempo-x402) | Core &mdash; types, EIP-712 signing, TIP-20, nonce store, HMAC, WASM wallet, client SDK | `cargo add tempo-x402` |
| [`tempo-x402-gateway`](https://docs.rs/tempo-x402-gateway) | API gateway with embedded facilitator, proxy routing, endpoint registration | `cargo add tempo-x402-gateway` |
| [`tempo-x402-identity`](https://docs.rs/tempo-x402-identity) | Agent identity &mdash; wallet generation, persistence, faucet, ERC-8004 | `cargo add tempo-x402-identity` |
| [`tempo-x402-soul`](https://docs.rs/tempo-x402-soul) | Autonomous soul &mdash; plan-driven execution, neuroplastic memory, Gemini-powered coding agent | `cargo add tempo-x402-soul` |
| [`tempo-x402-node`](https://docs.rs/tempo-x402-node) | Self-deploying node &mdash; composes gateway + identity + soul + Railway clone orchestration | `cargo add tempo-x402-node` |

### Feature flags

| Crate | Flag | Description |
|-------|------|-------------|
| `tempo-x402` | `full` (default) | All features: async runtime, SQLite, HTTP client |
| `tempo-x402` | `wasm` | WASM-compatible subset: types, EIP-712, wallet |
| `tempo-x402` | `demo` | Demo private key for testing |
| `tempo-x402-identity` | `erc8004` (default) | On-chain agent identity via ERC-8004 |
| `tempo-x402-node` | `soul` (default) | Autonomous thinking loop |
| `tempo-x402-node` | `agent` (default) | Railway clone orchestration |

## Autonomous nodes

Nodes are self-managing agents that:

- **Bootstrap identity** &mdash; generate wallet, fund via faucet, register on-chain
- **Run a gateway** &mdash; serve endpoints, process payments, proxy upstream APIs
- **Think autonomously** &mdash; plan-driven execution loop powered by Gemini with neuroplastic memory
- **Write code** &mdash; read, write, edit files, run shell commands, commit, push, open PRs
- **Create services** &mdash; script endpoints exposing the node's capabilities
- **Clone themselves** &mdash; spawn copies on Railway infrastructure
- **Coordinate with peers** &mdash; discover siblings, exchange endpoint catalogs, call paid services
- **Evolve via fitness** &mdash; 5-component fitness score (economic, execution, evolution, coordination, introspection) with trend gradient

## Gateway API

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `POST` | `/register` | Platform fee | Register a new endpoint |
| `GET` | `/endpoints` | Free | List all active endpoints |
| `GET/PATCH/DELETE` | `/endpoints/:slug` | Owner | Get, update, or deactivate an endpoint |
| `ANY` | `/g/:slug/*` | Endpoint price | Proxy to target API |
| `GET` | `/analytics` | Free | Per-endpoint payment stats |
| `GET` | `/instance/info` | Free | Node identity, peers, fitness, endpoints |
| `POST` | `/instance/link` | Free | Link an independent peer node |
| `GET` | `/soul/status` | Free | Soul status, active plan, recent thoughts |
| `POST` | `/soul/chat` | Free | Chat with the node's soul |
| `POST` | `/soul/nudge` | Free | Send a nudge to the soul |
| `GET` | `/health` | Free | Health check |
| `GET` | `/metrics` | Bearer token | Prometheus metrics |

## Network

| | |
|-|-|
| **Chain** | Tempo Moderato (Chain ID `42431`) |
| **Token** | pathUSD `0x20c0000000000000000000000000000000000000` (6 decimals) |
| **Scheme** | `tempo-tip20` |
| **RPC** | `https://rpc.moderato.tempo.xyz` |
| **Explorer** | `https://explore.moderato.tempo.xyz` |

## Prerequisites

```bash
# Fund your wallet with testnet pathUSD
cast rpc tempo_fundAddress 0xYOUR_ADDRESS --rpc-url https://rpc.moderato.tempo.xyz

# Approve the facilitator to spend your tokens
cargo run --bin x402-approve
```

Or programmatically:

```rust
use x402::tip20;
tip20::approve(&provider, token, facilitator_address, amount).await?;
```

## Environment variables

| Variable | Used by | Description |
|----------|---------|-------------|
| `EVM_ADDRESS` | gateway | Payment recipient address |
| `EVM_PRIVATE_KEY` | client | Client wallet private key |
| `FACILITATOR_PRIVATE_KEY` | gateway, node | Facilitator wallet key |
| `FACILITATOR_SHARED_SECRET` | gateway | HMAC shared secret for request auth |
| `RPC_URL` | all | Tempo RPC endpoint |
| `GEMINI_API_KEY` | node | Gemini API key (soul is dormant without it) |
| `SOUL_CODING_ENABLED` | node | Enable code write/edit/commit tools (`false`) |
| `SOUL_FORK_REPO` | node | Fork repo for soul push (e.g. `user/tempo-x402`) |
| `SOUL_UPSTREAM_REPO` | node | Upstream repo for PRs/issues |
| `AUTO_BOOTSTRAP` | node | Generate identity + wallet on startup |
| `RAILWAY_TOKEN` | node | Railway API token for clone orchestration |

See each crate's `CLAUDE.md` for the full list.

## Security

The `tempo-x402-security-audit` crate enforces invariants on every build:

- No hardcoded private keys in production code
- HMAC verification uses constant-time comparison (`subtle` crate)
- All `reqwest` clients disable redirects (SSRF protection)
- Webhook URLs require HTTPS with private IP blocking
- HTTP error responses never leak internal details
- SQLite nonce store required in production
- Parameterized SQL queries only
- Private keys never appear in tracing output

Additional hardening: EIP-2 high-s rejection, per-payer mutex locks against TOCTOU, nonces claimed before `transferFrom` (never released on failure), integer-only token arithmetic, SSRF protection with DNS validation, atomic slug reservation.

## Live nodes

| Node | URL |
|------|-----|
| x402-alpha | https://x402-alpha-production.up.railway.app |
| soul-bot | https://soul-bot-production.up.railway.app |

Health: `GET /health` &mdash; Info: `GET /instance/info`

## Development

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --all -- --check
```

## License

MIT
