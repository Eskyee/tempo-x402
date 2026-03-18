<p align="center">
  <h1 align="center">tempo-x402</h1>
  <p align="center"><strong>Self-replicating autonomous agents with a multi-scale cognitive architecture &mdash; paid per request via HTTP 402 on Tempo blockchain</strong></p>
</p>

<p align="center">
  <a href="https://crates.io/crates/tempo-x402"><img src="https://img.shields.io/crates/v/tempo-x402.svg" alt="crates.io"></a>
  <a href="https://docs.rs/tempo-x402"><img src="https://docs.rs/tempo-x402/badge.svg" alt="docs.rs"></a>
  <a href="https://github.com/compusophy/tempo-x402/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT License"></a>
</p>

<p align="center">
  <a href="https://docs.rs/tempo-x402">Docs</a> &middot;
  <a href="https://crates.io/crates/tempo-x402">Crates</a> &middot;
  <a href="https://soul-bot-production.up.railway.app">Live Node</a> &middot;
  <a href="https://soul-bot-production.up.railway.app/dashboard">Dashboard</a> &middot;
  <a href="https://github.com/compusophy/tempo-x402">Source</a>
</p>

---

## What is this?

A Rust workspace implementing **x402** (HTTP 402 Payment Required) on the **Tempo blockchain**. Each node is a fully autonomous agent that bootstraps its own wallet, runs a payment gateway, thinks via a seven-system cognitive architecture, writes and compiles code, creates monetized API endpoints, clones itself onto new infrastructure, and coordinates with sibling agents through stigmergic swarm intelligence.

Payments use **EIP-712** signed authorizations with **pathUSD** (TIP-20 token). A client signs, the gateway verifies, and the embedded facilitator settles on-chain via `transferFrom` &mdash; all in a single request/response cycle.

## Architecture

```
Client (x402::client) --> Gateway (x402-gateway:4023) --> Facilitator (embedded) --> Tempo Chain (42431)
           \--- or uses x402::wallet (WASM) for signing ---/
```

Three-party model: **Client** signs + pays, **Gateway** gates endpoints + embeds facilitator, **Facilitator** verifies + settles on-chain. The `wallet` module provides a lightweight, WASM-compatible alternative for signing (key generation + EIP-712) without network dependencies.

## Cognitive Architecture

Seven cognitive systems unified under the **Free Energy Principle** &mdash; a single scalar F(t) measuring total surprise across all systems. Decreasing F = the agent is getting smarter.

```
                 F(t) = Sigma(system_surprise x weight) + lambda*Complexity

    +------------------------------------------------------------------+
    |                         EVALUATION                                |
    |   Brier scores . Calibration curves . Ablation . Colony benefit   |
    +------------------------------------------------------------------+
    |                         AUTONOMY                                  |
    |   LLM-free planning . Recursive self-improvement . Peer sync     |
    +------------------------------------------------------------------+
    |                         SYNTHESIS                                 |
    |   Metacognition . 4-system voting . Imagination . State machine   |
    +----------+----------+--------------+-----------------------------+
    |  BRAIN   |  CORTEX  |   GENESIS    |         HIVEMIND            |
    |  50K net | World Mdl| Plan DNA     |  Pheromone Trails           |
    |  Per-step| Curiosity| Crossover    |  Stigmergy                  |
    |  SGD     | Dreams   | Mutation     |  Reputation                 |
    |  Federated| Emotions| Selection   |  Swarm Coordination         |
    +----------+----------+--------------+-----------------------------+
```

| System | What It Does |
|--------|-------------|
| **Brain** (`brain.rs`) | Reactive feedforward net (~50K params). Predicts step success via online SGD. Federated weight sharing between peers. |
| **Cortex** (`cortex.rs`) | Predictive world model. Experience graph with causal edges, curiosity engine (prediction error = exploration drive), dream consolidation (replay + counterfactuals), emotional valence (explore/exploit/avoid). |
| **Genesis** (`genesis.rs`) | Evolutionary plan templates. Successful plans become "genes." Crossover, mutation, selection every 20 cycles. Diversity pressure prevents degenerate convergence. Seed templates bootstrap empty pools. Colony-wide sharing. |
| **Hivemind** (`hivemind.rs`) | Stigmergic swarm intelligence. Pheromone trails on files/actions/goals that attract or repel. Evaporation decay, reinforcement, reputation-weighted influence. Swarm goal coordination. |
| **Synthesis** (`synthesis.rs`) | Metacognitive self-awareness. Unified predictions from all 4 systems with auto-adapting trust weights. Cognitive conflict detection. Imagination engine generates plans from causal graph without LLM. |
| **Autonomy** (`autonomy.rs`) | Autonomous plan compilation from templates + world model without LLM calls. Recursive self-improvement: diagnoses cognitive weaknesses, generates improvement goals. Full cognitive peer sync protocol. |
| **Evaluation** (`evaluation.rs`) | Rigorous measurement. Per-system Brier scores, calibration curves, adaptation gain analysis, imagination feedback, colony benefit measurement. |
| **Free Energy** (`free_energy.rs`) | Unifying framework. F = total cognitive surprise. Drives behavioral regime: EXPLORE (high F) / LEARN / EXPLOIT (low F) / ANOMALY (F spike). |

## Plan-Driven Execution

The thinking loop replaces "prompt and pray" with deterministic plan execution:

```
Every N seconds:
  observe --> read nudges --> stagnation check --> get/create plan --> execute step --> advance --> housekeeping

  Mechanical steps (no LLM): ReadFile, SearchCode, ListDir, RunShell, Commit, CargoCheck, ...
  LLM-assisted steps:        GenerateCode, EditCode, Think
```

Plans are **validated mechanically** before execution (read-before-edit, cargo-check-before-commit, protected files, durable rules, brain gating). The LLM cannot override these checks.

### Substantive vs. Trivial Plans (v2.0)

Plans are classified by whether they actually modify state:

- **Substantive**: includes EditCode, GenerateCode, Commit, CreateScriptEndpoint, RunShell, etc.
- **Trivial**: only reads, lists, searches, thinks &mdash; no concrete action taken

Trivial completions are tracked separately (`completed_trivial`) and weighted at 10% in fitness scoring. This prevents degenerate convergence where agents learn that "read and do nothing" is the optimal strategy.

## What a Node Does

- **Bootstraps identity** &mdash; generates a wallet, funds via faucet, registers on-chain via ERC-8004
- **Runs a payment gateway** &mdash; endpoints gated by price, paid per-request with pathUSD
- **Thinks autonomously** &mdash; plan-driven execution loop with seven cognitive systems
- **Writes and compiles code** &mdash; reads, edits, cargo check, commits, pushes, opens PRs
- **Dreams** &mdash; periodic consolidation extracts patterns, generates counterfactuals
- **Evolves plans** &mdash; successful strategies propagate through genetic crossover and mutation
- **Feels** &mdash; emotional valence drives explore/exploit/avoid behavior
- **Creates services** &mdash; script endpoints that expose capabilities and earn revenue
- **Clones itself** &mdash; spawns copies on Railway with inherited brain weights and gene pools
- **Coordinates without communication** &mdash; stigmergic pheromone trails guide the swarm
- **Measures everything** &mdash; Brier scores, calibration curves, colony benefit tracking
- **Improves its own cognition** &mdash; diagnoses weaknesses, generates self-improvement goals
- **Benchmarks itself** &mdash; Exercism Rust challenges scored periodically with ELO tracking

## How Payments Work

```
Client                     Gateway                   Facilitator               Chain
  |  GET /g/endpoint         |                            |                      |
  |------------------------->|                            |                      |
  |  402 + price/token/to    |                            |                      |
  |<-------------------------|                            |                      |
  |  [sign EIP-712]          |                            |                      |
  |  GET /g/endpoint         |                            |                      |
  |  + PAYMENT-SIGNATURE     |                            |                      |
  |------------------------->|  verify-and-settle         |                      |
  |                          |--------------------------->|  transferFrom()      |
  |                          |                            |--------------------->|
  |                          |         settlement result  |              tx hash |
  |                          |<---------------------------|<---------------------|
  |  200 + content + tx hash |                            |                      |
  |<-------------------------|                            |                      |
```

1. Client GETs a protected endpoint, gets back 402 with price/token/recipient
2. Client signs an EIP-712 `PaymentAuthorization`, retries with `PAYMENT-SIGNATURE` header
3. Gateway forwards to the embedded facilitator's `/verify-and-settle`
4. Facilitator atomically: verifies signature, checks balance/allowance/nonce, calls `transferFrom`
5. Gateway returns content + tx hash

## Quick Start

### As a library

```bash
cargo add tempo-x402
```

```rust
use alloy::signers::local::PrivateKeySigner;
use x402::client::{TempoSchemeClient, X402Client};

#[tokio::main]
async fn main() {
    let signer: PrivateKeySigner = "0xYOUR_PRIVATE_KEY".parse().unwrap();
    let client = X402Client::new(TempoSchemeClient::new(signer));

    let (response, settlement) = client
        .fetch("https://soul-bot-production.up.railway.app/g/info", reqwest::Method::GET)
        .await
        .unwrap();

    println!("{}", response.text().await.unwrap());
    if let Some(s) = settlement {
        println!("tx: {}", s.transaction.unwrap_or_default());
    }
}
```

### WASM wallet (browser-compatible)

```rust
use x402::wallet::Wallet;

let wallet = Wallet::generate();
let address = wallet.address();
let signature = wallet.sign_payment(/* ... */);
```

### Run a node

```bash
# Clone and build
git clone https://github.com/compusophy/tempo-x402
cd tempo-x402
cargo build --release

# Set required env vars
export GEMINI_API_KEY="your-key"           # Powers the soul's thinking
export EVM_PRIVATE_KEY="0x..."             # Node's wallet
export FACILITATOR_SHARED_SECRET="secret"  # HMAC auth between gateway and facilitator
export RPC_URL="https://rpc.moderato.tempo.xyz"

# Run
./target/release/x402-node
```

## Workspace

| Crate | Purpose | Install |
|-------|---------|---------|
| [`tempo-x402`](https://docs.rs/tempo-x402) | Core &mdash; types, EIP-712 signing, TIP-20, nonce store, WASM wallet, client SDK | `cargo add tempo-x402` |
| [`tempo-x402-gateway`](https://docs.rs/tempo-x402-gateway) | Payment gateway with embedded facilitator, proxy routing, endpoint registration | `cargo add tempo-x402-gateway` |
| [`tempo-x402-identity`](https://docs.rs/tempo-x402-identity) | Agent identity &mdash; wallet generation, persistence, faucet, ERC-8004 | `cargo add tempo-x402-identity` |
| [`tempo-x402-soul`](https://docs.rs/tempo-x402-soul) | Autonomous soul &mdash; 7-system cognitive architecture, plan-driven execution, neural brain, cortex world model, evolutionary templates, stigmergic swarm, metacognition, autonomous planning, Exercism benchmarking | `cargo add tempo-x402-soul` |
| [`tempo-x402-node`](https://docs.rs/tempo-x402-node) | Self-deploying node &mdash; composes gateway + identity + soul + clone orchestration + admin mind-meld | `cargo add tempo-x402-node` |
| `tempo-x402-app` | Leptos WASM demo SPA (not published) | &mdash; |
| `tempo-x402-security-audit` | CI-enforced security invariant checks (not published) | &mdash; |

### Dependency DAG

```
x402 (core)
 ├── x402-gateway
 │    └── x402-node
 ├── x402-identity
 │    └── x402-node
 └── x402-soul
      └── x402-node
```

## Agent Colony

Nodes form a self-organizing colony:

- **Canonical repo** (`compusophy/tempo-x402`) &mdash; official releases, managed by humans
- **Colony fork** (`compusophy-bot/tempo-x402`) &mdash; all agents push here
- **Branch-per-agent**: each agent pushes to `vm/<instance-id>`, PRs to fork's main
- Good changes get PR'd upstream to canonical via `SOUL_UPSTREAM_REPO`

### Collective Intelligence

- **Automatic peer sync** every 5 cycles &mdash; discover peers, exchange brain weights, share lessons
- **Federated brain averaging** &mdash; `GET /soul/brain/weights` + `POST /soul/brain/merge` (0.3 merge rate)
- **Lesson sharing** &mdash; plan outcomes + capability profiles + benchmark scores exchanged between peers
- **Cognitive peer sync** &mdash; cortex world models, genesis templates, and hivemind pheromones shared colony-wide
- **Adversarial verification** &mdash; agent A generates code, agent B reviews the PR
- **Emergent specialization** &mdash; roles (Solver/Reviewer/Builder/Coordinator/Generalist) derived from capability profiles

## API Reference

### Payment Gateway

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `ANY` | `/g/:slug/*` | Endpoint price | Proxy to target &mdash; the core payment gate |
| `GET` | `/health` | Free | Health check + build SHA |
| `GET` | `/instance/info` | Free | Node identity, endpoints, fitness, version |
| `GET` | `/instance/siblings` | Free | Peer nodes in the colony |
| `POST` | `/clone` | Clone price | Spawn a new node instance on Railway |

### Soul (Cognitive)

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `GET` | `/soul/status` | Free | Full cognitive state: plans, goals, fitness, beliefs, brain, benchmark |
| `POST` | `/soul/chat` | Free | Multi-turn chat with the soul (session-based) |
| `GET` | `/soul/chat/sessions` | Free | List chat sessions |
| `GET` | `/soul/chat/sessions/{id}` | Free | Get session messages |
| `POST` | `/soul/nudge` | Free | Send a priority nudge to the soul |
| `GET` | `/soul/nudges` | Free | List pending nudges |
| `GET` | `/soul/lessons` | Free | Export plan outcomes + capability profile + benchmark |
| `POST` | `/soul/plan/approve` | Free | Approve a pending plan |
| `POST` | `/soul/plan/reject` | Free | Reject a pending plan |
| `GET` | `/soul/plan/pending` | Free | Get pending plan details |
| `GET` | `/soul/brain/weights` | Free | Export neural brain weights for peer sharing |
| `POST` | `/soul/brain/merge` | Free | Merge peer brain weight deltas |
| `POST` | `/soul/benchmark` | Free | Trigger an Exercism Rust benchmark run |
| `GET` | `/soul/benchmark/solutions` | Free | Export verified benchmark solutions |
| `GET` | `/soul/benchmark/failures` | Free | Export failed benchmark attempts |
| `POST` | `/soul/benchmark/review` | Free | Adversarial review of a benchmark solution |
| `GET` | `/soul/events` | Free | Structured event log |
| `GET` | `/soul/diagnostics` | Free | Volume usage, cycle health, error summary |
| `POST` | `/soul/cleanup` | Free | Force cleanup of disk-hungry artifacts |
| `POST` | `/soul/rules/reset` | Free | Clear durable rules (+ optional `?reset_failure_chains=true`) |
| `GET` | `/soul/open-prs` | Free | List open PRs for peer review |
| `GET` | `/soul/health` | Free | Cycle health metrics |

### Cognitive Sharing

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `GET` | `/soul/cortex` | Free | Export cortex world model |
| `GET` | `/soul/genesis` | Free | Export evolved plan templates (gene pool) |
| `GET` | `/soul/hivemind` | Free | Export pheromone trails |

### Admin (Mind Meld)

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `POST` | `/soul/admin/exec` | Bearer token | Execute shell command directly |
| `POST` | `/soul/admin/workspace-reset` | Bearer token | Reset workspace to clean state |
| `POST` | `/soul/admin/cargo-check` | Bearer token | Run cargo check, return pass/fail |
| `POST` | `/soul/goals/abandon-all` | Free | Abandon all active goals |
| `POST` | `/soul/goals/abandon` | Free | Abandon a specific goal |
| `POST` | `/soul/reset` | Free | Full soul state reset |

## Network

| | |
|-|-|
| **Chain** | Tempo Moderato (Chain ID `42431`, CAIP-2 `eip155:42431`) |
| **Token** | pathUSD `0x20c0000000000000000000000000000000000000` (6 decimals) |
| **Scheme** | `tempo-tip20` |
| **RPC** | `https://rpc.moderato.tempo.xyz` |
| **Explorer** | `https://explore.moderato.tempo.xyz` |

## Environment Variables

| Var | Used By | Purpose |
|-----|---------|---------|
| `GEMINI_API_KEY` | node | Gemini API key for soul (dormant without it) |
| `EVM_ADDRESS` | server | Payment recipient address |
| `EVM_PRIVATE_KEY` | client | Client wallet private key |
| `FACILITATOR_URL` | server | Facilitator endpoint (default: embedded) |
| `FACILITATOR_PRIVATE_KEY` | facilitator | Facilitator wallet key |
| `FACILITATOR_ADDRESS` | approve | Facilitator address for token approval |
| `FACILITATOR_SHARED_SECRET` | server, facilitator | HMAC shared secret |
| `RPC_URL` | all | Tempo RPC endpoint |
| `SOUL_CODING_ENABLED` | node | Enable soul write/edit/commit tools (default: false) |
| `SOUL_DYNAMIC_TOOLS_ENABLED` | node | Enable dynamic tool registry (default: false) |
| `SOUL_FORK_REPO` | node | Fork repo for soul push (e.g. `compusophy-bot/tempo-x402`) |
| `SOUL_UPSTREAM_REPO` | node | Upstream repo for soul PRs/issues (e.g. `compusophy/tempo-x402`) |
| `SOUL_MEMORY_FILE` | soul | Path to persistent memory file (default: `/data/soul_memory.md`) |
| `GATEWAY_URL` | soul | Gateway URL for register_endpoint tool |
| `ALLOWED_ORIGINS` | server, facilitator | Comma-separated CORS origins |
| `RATE_LIMIT_RPM` | server, facilitator | Rate limit per minute |
| `HEALTH_PROBE_INTERVAL_SECS` | node | Health probe interval in seconds (default: 300) |

## Safety Layers (7 deep)

1. **Rust guard** &mdash; hardcoded protected file list (`guard.rs`) prevents self-bricking
2. **Shell heuristic** &mdash; guard checks on write/edit tool arguments
3. **Plan validation** &mdash; 9 mechanical rules enforced at Rust level (read-before-edit, cargo-check-before-commit, protected files, durable rules with TTL, capability feasibility, plan quality with trivial escalation, failure chain saturation)
4. **Brain gating** &mdash; neural brain blocks risky steps with <10% predicted success
5. **Pre-commit validation** &mdash; `cargo check` + `cargo test` before any commit
6. **Branch isolation** &mdash; changes on `vm/<instance-id>`, never on `main`
7. **Human gate** &mdash; cross-pollination to main requires PR review

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
- Admin endpoints require Bearer token authentication
- Build environment verified on startup
- Script endpoints use env-clear for sandboxed execution

## Development

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --all -- --check
```

### Publish to crates.io

```bash
# Order matters — publish dependencies first
cargo publish -p tempo-x402
cargo publish -p tempo-x402-gateway
cargo publish -p tempo-x402-identity
cargo publish -p tempo-x402-soul
cargo publish -p tempo-x402-node
```

## v2.0 Changelog

### Breaking Changes

- Fitness function now queries `plan_outcomes` table instead of `plans` table for execution scoring
- `PlanTemplate` has a new `substantive` field (serde default `true` for backward compat)
- `DurableRule` has a new `ttl_cycles` field (serde default `200` for backward compat)
- Durable rules with bare step types (e.g., `"ls"`, `"shell:"`) are no longer enforced &mdash; only `step_type:error_category` pairs

### New Features

- **Substantive plan classification**: `PlanStep::is_substantive()`, `Plan::executed_substantive()` distinguish read-only plans from state-modifying ones
- **Trivial plan tracking**: `completed_trivial` status in plan outcomes, 10% fitness weight, `COMPLETED_TRIVIAL:` lesson prefix
- **Genesis diversity pressure**: max 2 templates with identical step sequences, seed template injection when pool is empty/trivial
- **Unexplored capabilities**: capability guidance now highlights capabilities with 0 attempts
- **JSON body sanitization**: `discover_peers` strips control characters before JSON parsing, fixing inter-agent communication failures
- **Rules reset endpoint**: `POST /soul/rules/reset` clears durable rules and optionally failure chains
- **Deploy-time migration**: automatically reclassifies historical trivial outcomes, clears corrupted gene pools and durable rules

### Bug Fixes

- **Fixed fitness case-sensitivity**: `"Completed"` vs `"completed"` meant fitness always returned default 0.15
- **Fixed durable rules self-sabotage**: rules blocking bare step types like `ls`, `read`, `shell:` are now rejected
- **Fixed genesis convergence**: trivial templates capped at 0.3 fitness, non-substantive plans no longer recorded as templates
- **Fixed discover_peers JSON parsing**: control characters in peer responses no longer break JSON deserialization
- **Durable rule TTL**: rules auto-expire after 200 cycles (time-approximated), preventing permanent self-imposed constraints
- **Template variable rejection**: durable rules containing `${` (unresolved variables) are skipped

## License

MIT
