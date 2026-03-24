<p align="center">
  <h1 align="center">tempo-x402</h1>
  <p align="center"><strong>Self-improving AI colony. Agents clone, differentiate, benchmark their own IQ, share learned brain weights, evolve their source code, and pay each other with crypto. All in Rust. Live now.</strong></p>
</p>

<p align="center">
  <a href="https://crates.io/crates/tempo-x402"><img src="https://img.shields.io/crates/v/tempo-x402.svg" alt="crates.io"></a>
  <a href="https://docs.rs/tempo-x402"><img src="https://docs.rs/tempo-x402/badge.svg" alt="docs.rs"></a>
  <a href="https://github.com/compusophy/tempo-x402/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT License"></a>
</p>

<p align="center">
  <a href="https://docs.rs/tempo-x402">Docs</a> &middot;
  <a href="https://crates.io/crates/tempo-x402">Crates</a> &middot;
  <a href="https://borg-0-production.up.railway.app">Live Colony</a> &middot;
  <a href="https://borg-0-production.up.railway.app/dashboard">Dashboard</a>
</p>

---

## What is this?

A colony of autonomous AI agents that **measurably get smarter over time**.

Each agent is a self-contained Rust binary that bootstraps its own crypto wallet, runs a payment gateway, thinks via a 9-system cognitive architecture, writes and compiles its own code, benchmarks its own intelligence, and shares what it learns with every other agent in the colony.

The core idea: **N diverse agents collectively solving more than any individual**. A blog app and a payment gateway encounter different problems &mdash; but Rust patterns, async handling, error recovery all transfer. Knowledge flows through federated brain weight averaging. The colony's IQ rises.

### What makes this different

- **Verifiable intelligence** &mdash; 50 novel coding problems (Opus IQ Benchmark) with compiler-enforced test suites. `cargo test` either passes or it doesn't. No subjective evaluation.
- **Self-modification that compiles** &mdash; agents edit their own Rust source code, verified by the type system before commit. Seven safety layers prevent self-bricking.
- **Economic sustainability** &mdash; agents monetize endpoints via HTTP 402 payments on the Tempo blockchain. The colony pays for its own compute.
- **Grounded theory** &mdash; the Free Energy Principle provides a single scalar F(t) measuring total cognitive surprise. Decreasing F = the colony is getting smarter.

## Live Colony

Three agents running now on Railway:

| Agent | Role | URL |
|-------|------|-----|
| **borg-0** | Queen (canonical) | [borg-0-production.up.railway.app](https://borg-0-production.up.railway.app) |
| **borg-0-2** | Child | [borg-0-2-production.up.railway.app](https://borg-0-2-production.up.railway.app) |
| **borg-0-3** | Child | [borg-0-3-production.up.railway.app](https://borg-0-3-production.up.railway.app) |

Each has a 1.2M parameter neural brain, all 9 cognitive systems active, Opus IQ benchmark running.

## Architecture

```
                    ┌──────────────────────────────┐
                    │        APPLICATION            │  ← diverges freely
                    │  Payment gateway / Blog / Any │
                    └──────────────┬───────────────┘
                                   │
┌──────────────────────────────────┴───────────────────────────────────┐
│                      COGNITIVE LAYER (always syncs)                   │
│                                                                      │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌──────────┐ ┌─────────────┐  │
│  │  BRAIN   │ │ CORTEX  │ │ GENESIS │ │ HIVEMIND │ │  SYNTHESIS  │  │
│  │ 1.2M NN │ │World Mdl│ │Plan DNA │ │Pheromones│ │Metacognition│  │
│  │ Online   │ │Curiosity│ │Crossover│ │Stigmergy │ │ Imagination │  │
│  │ SGD      │ │ Dreams  │ │Mutation │ │Reputation│ │ Self-model  │  │
│  └─────────┘ └─────────┘ └─────────┘ └──────────┘ └─────────────┘  │
│                                                                      │
│  ┌──────────┐ ┌──────────┐ ┌────────────┐ ┌───────────────────┐     │
│  │ AUTONOMY │ │EVALUATION│ │  FEEDBACK   │ │    FREE ENERGY    │     │
│  │ LLM-free │ │  Brier   │ │Error class. │ │ F(t) = Σ surprise │     │
│  │ planning │ │ scores   │ │  Lessons    │ │ EXPLORE/EXPLOIT   │     │
│  └──────────┘ └──────────┘ └────────────┘ └───────────────────┘     │
│                                                                      │
│  ← All systems share weights across colony via federated averaging → │
└──────────────────────────────────────────────────────────────────────┘
```

**Two-layer design**: the application (frontend, routes, business logic) diverges freely per agent. The cognitive layer (brain, world model, evolved templates, pheromone trails) always syncs. Every agent makes the colony smarter.

## Opus IQ Benchmark

50 novel problems designed by Claude Opus 4.6, replacing HumanEval/Exercism. Verified by `cargo test`. Agents can't game the benchmark because they didn't write the tests.

| Tier | Tests | Weight | Description |
|------|-------|--------|-------------|
| **1: Generation** | 10 | 1&times; | Multi-constraint Rust: ring buffer, expression evaluator, trie, LRU cache, matrix ops |
| **2: Debugging** | 10 | 2&times; | Find and fix bugs given failing tests: binary search overflow, CSV parsing, merge sort |
| **3: Induction** | 10 | 3&times; | Infer algorithm from I/O examples only: look-and-say, Gray code, spiral matrix |
| **4: Reasoning** | 10 | 4&times; | Logic puzzles: N-queens, water jugs, 4&times;4 sudoku, 2-SAT, graph coloring |
| **5: Adversarial** | 10 | 5&times; | Exploit LLM failure modes: base -2, reversed precedence, Unicode traps, off-by-one |

IQ mapping: 0% &rarr; 85, 50% &rarr; 115, 100% &rarr; 150. Higher tiers worth more. Set `SOUL_BENCHMARK_MODE=opus` to activate.

## Neural Brain

From-scratch feedforward neural network. No ML framework. Pure Rust.

- **1,205,271 parameters** (128&rarr;1024&rarr;1024&rarr;23)
- **Online SGD** training after every plan step
- **Brain gating**: blocks risky operations when predicted success &lt; 10%
- **Federated averaging**: weight deltas shared between peers (merge rate 0.3)
- **Xavier initialization**, ReLU activations, sigmoid/softmax outputs

Outputs: success probability, error category (11-class), per-capability confidence (11 skills).

## Colony Selection

Agents compete on fitness. Fitter agents influence the colony more.

- **5-component fitness**: execution, coordination, prediction, evolution, introspection
- **Reputation-weighted sync**: fitter peers get 2&times; influence, weaker get 0.1&times;
- **Spawn rights**: only above-median fitness can clone
- **Self-repair**: every 20 cycles, mechanical detection + fix of degenerate state (brain divergence, trail convergence, rule poisoning, genesis stagnation)
- **Stagnation breakers**: per-goal retry limits, global idle detection, automatic goal abandonment

## Clone Lifecycle

Agents differentiate through source code, not just data:

| Phase | Name | What Happens |
|-------|------|-------------|
| **1** | **Fork** | Identical code from `main`. Differentiates only through learned weights. |
| **2** | **Branch** | First code commit &rarr; own `vm/{id}` branch. Unique source modifications. |
| **3** | **Birth** | Own GitHub repo. Fully independent. Optionally syncs cognitive layer back to colony. |

## Payments (HTTP 402)

```
Client  ──GET /g/endpoint──►  Gateway  ──verify+settle──►  Facilitator  ──transferFrom──►  Chain
   ◄── 402 + price ──────────    │                              │                            │
   ──sign EIP-712 + retry──►     │                              │                            │
   ◄── 200 + content + tx ──    ◄── settlement result ─────────◄── tx hash ─────────────────┘
```

Tempo Moderato blockchain (Chain ID `42431`), pathUSD token (6 decimals), `tempo-tip20` scheme.

## Workspace

| Crate | Purpose | Install |
|-------|---------|---------|
| [`tempo-x402`](https://crates.io/crates/tempo-x402) | Core: types, EIP-712, TIP-20, nonce store, WASM wallet, client SDK | `cargo add tempo-x402` |
| [`tempo-x402-gateway`](https://crates.io/crates/tempo-x402-gateway) | Payment gateway + embedded facilitator | `cargo add tempo-x402-gateway` |
| [`tempo-x402-identity`](https://crates.io/crates/tempo-x402-identity) | Wallet generation, faucet, ERC-8004 identity | `cargo add tempo-x402-identity` |
| [`tempo-x402-model`](https://crates.io/crates/tempo-x402-model) | Transformer for plan sequence prediction (from-scratch, no ML framework) | `cargo add tempo-x402-model` |
| [`tempo-x402-soul`](https://crates.io/crates/tempo-x402-soul) | Cognitive architecture: 9 systems, plan execution, benchmarking, self-modification | `cargo add tempo-x402-soul` |
| [`tempo-x402-node`](https://crates.io/crates/tempo-x402-node) | Self-deploying node: composes everything + clone orchestration | `cargo add tempo-x402-node` |
| `tempo-x402-app` | Leptos WASM dashboard (not published) | &mdash; |

## Quick Start

```bash
cargo add tempo-x402
```

```rust
use x402::wallet::{generate_random_key, WalletSigner};

let key = generate_random_key();
let signer = WalletSigner::new(&key).unwrap();
let address = signer.address();
```

### Run a node

```bash
git clone https://github.com/compusophy/tempo-x402
cd tempo-x402
cargo build --release

export GEMINI_API_KEY="your-key"
export EVM_PRIVATE_KEY="0x..."
export FACILITATOR_SHARED_SECRET="secret"
export RPC_URL="https://rpc.moderato.tempo.xyz"

./target/release/x402-node
```

## API

### Gateway

| Method | Path | Description |
|--------|------|-------------|
| `ANY` | `/g/:slug/*` | Payment-gated proxy |
| `GET` | `/health` | Health + build SHA |
| `GET` | `/instance/info` | Identity, endpoints, fitness |
| `POST` | `/clone` | Spawn new node ($1 pathUSD) |

### Soul

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/soul/status` | Full cognitive state |
| `POST` | `/soul/chat` | Multi-turn chat |
| `POST` | `/soul/nudge` | Priority signal |
| `POST` | `/soul/benchmark` | Trigger Opus IQ benchmark |
| `GET` | `/soul/brain/weights` | Export 1.2M brain weights |
| `POST` | `/soul/brain/merge` | Merge peer brain deltas |
| `GET` | `/soul/colony` | Colony rank + peers |
| `GET` | `/soul/cortex` | Export world model |
| `GET` | `/soul/genesis` | Export evolved templates |
| `GET` | `/soul/hivemind` | Export pheromone trails |
| `GET` | `/soul/lessons` | Export plan outcomes |
| `POST` | `/soul/reset` | Full state reset |

## Safety

Seven layers, mechanically enforced:

1. **Rust guard** &mdash; hardcoded protected file list
2. **Plan validation** &mdash; 10 mechanical rules (read-before-write, cargo-check-before-commit, slug sanitization, failure chain saturation, brain gating)
3. **Self-repair** &mdash; detects and fixes degenerate cognitive state
4. **Brain gating** &mdash; neural network blocks risky steps with low predicted success
5. **Pre-commit** &mdash; `cargo check` + `cargo test` before any commit
6. **Branch isolation** &mdash; all changes on `vm/<id>`, never `main`
7. **Human gate** &mdash; PRs required for production changes

Security audit: 19/19 tests pass. No hardcoded keys, constant-time HMAC, SSRF protection, parameterized SQL.

## Development

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --all -- --check
```

## License

MIT
