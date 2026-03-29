# tempo-x402 — Project Context

## What This Is

Autonomous AI colony on the Tempo blockchain. Self-replicating agents that clone, evolve source code, benchmark IQ, share neural weights, and pay each other via HTTP 402. Agents write Rust programs, compile to WASM cartridges, and deploy instantly.

Rust workspace. 9 crates, ~70K lines. Published as `tempo-x402`, `tempo-x402-cartridge`, `tempo-x402-gateway`, `tempo-x402-identity`, `tempo-x402-model`, `tempo-x402-soul`, `tempo-x402-node` on crates.io.

## Architecture

```
Client ──► Gateway (4023) ──► Facilitator (embedded) ──► Tempo Chain (42431)
               │
               ├── Identity (wallet bootstrap + faucet + ERC-8004)
               ├── Soul (9-system cognitive architecture, Gemini-powered)
               ├── Cartridge Engine (wasmtime WASM sandbox runtime)
               └── Clone Orchestrator (Railway self-replication + stem cell differentiation)
```

Two-layer design: **Application layer** (routes, frontend, cartridges) diverges per agent. **Cognitive layer** (brain, cortex, genesis, hivemind, synthesis, autonomy, evaluation, feedback, free energy) always syncs across the colony.

**Stem cell model**: Each clone gets its own GitHub repo. Code diverges independently. Good changes flow upstream via PRs.

## Workspace

```
crates/
├── tempo-x402/                # core: types, EIP-712, TIP-20, nonce store, WASM wallet, client SDK
├── tempo-x402-gateway/        # API proxy + embedded facilitator + payment middleware
├── tempo-x402-identity/       # wallet generation, faucet, on-chain ERC-8004 identity
├── tempo-x402-model/          # from-scratch transformer for plan sequence prediction
├── tempo-x402-cartridge/      # WASM cartridge runtime (wasmtime) — sandboxed app execution
│   └── src/                   # engine, ABI, compiler, manifest, error
├── tempo-x402-soul/           # 9-system cognitive architecture + plan execution + benchmarking
│   ├── src/tools/             # tool executor split by domain (10 files incl. cartridges)
│   ├── src/thinking/          # thinking loop split (7 files)
│   ├── src/db/                # SQLite CRUD split by domain (13 files)
│   └── src/opus_bench/        # 50 benchmark problems split by tier (6 files)
├── tempo-x402-node/           # self-deploying binary: gateway + identity + soul + cloning
│   └── src/routes/            # soul/ (9 files), cartridges.rs, scripts.rs, clone.rs, etc.
├── tempo-x402-app/            # Leptos WASM dashboard (not published)
│   ├── src/components/        # UI components (8 files)
│   └── src/cartridges.rs      # Cartridge browser + test console
└── tempo-x402-security-audit/ # 19 security invariant tests (not published)
```

Dependency DAG: `x402 → gateway → node`, `x402 → identity → node`, `x402 → soul → node`, `x402 → model → soul`, `cartridge → soul, node`.

Each crate has its own `CLAUDE.md` with local context. **Read that first when working in a crate.**

## Live Colony

| Agent | Domain | Role |
|-------|--------|------|
| **borg-0** | `borg-0-production.up.railway.app` | Queen (canonical, pushes to colony fork) |
| **borg-0-2** | `borg-0-2-production.up.railway.app` | Clone (own repo, independent evolution) |

Colony repos:
- `compusophy/tempo-x402` — canonical (user + Claude Code only)
- `compusophy-bot/tempo-x402` — colony baseline (queen pushes here)
- `compusophy-bot/{designation}` — clone's own repo (stem cell model)

## WASM Cartridges

Agents write Rust → compile to WASM → deploy instantly at `/c/{slug}` with payment gate.

- **Create**: `create_cartridge(slug, source_code)` — scaffolds Rust project
- **Compile**: `compile_cartridge(slug)` — `cargo build --target wasm32-wasip1`
- **Test**: `test_cartridge(slug, method, path, body)` — runs in wasmtime sandbox
- **Serve**: `GET/POST /c/{slug}` — x402 payment gated
- **Studio**: `/cartridges` page with browser + test console
- **Safety**: 64MB memory, fuel CPU limit, 30s timeout, no filesystem access

## Agent Discipline

- **Benchmark-driven commit gate**: Agent can't commit again until benchmark measures IQ delta of last commit. No hardcoded timers — clears when system confirms measurement.
- **Cumulative destruction guard**: Tracks total file changes over 24h against deploy baseline. Blocks >70% cumulative deletion (prevents incremental lobotomy).
- **Post-commit benchmark**: Every commit forces benchmark run. Brain trains on the delta.

## Chain

- **Chain**: Tempo Moderato, Chain ID `42431`, CAIP-2 `eip155:42431`
- **Token**: pathUSD `0x20c0000000000000000000000000000000000000` (6 decimals)
- **Scheme**: `tempo-tip20`
- **RPC**: `https://rpc.moderato.tempo.xyz`

## Key Environment Variables

| Var | Used By | Purpose |
|-----|---------|---------|
| `GEMINI_API_KEY` | node | Gemini API key for soul (dormant without it) |
| `SOUL_CODING_ENABLED` | node | Enable write/edit/commit tools |
| `SOUL_FORK_REPO` | node | Fork repo for agent push |
| `SOUL_UPSTREAM_REPO` | node | Upstream repo for PRs |
| `SOUL_BENCHMARK_MODE` | node | `opus` or `exercism` — propagated to clones |
| `SOUL_MEMORY_FILE` | soul | Persistent memory (default: `/data/soul_memory.md`) |
| `RAILWAY_TOKEN` | node | Railway API for clone orchestration |
| `METRICS_TOKEN` | node | /metrics endpoint auth — propagated to clones |

## Commands

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --all
```

## Publishing

Publish in dependency order: `x402` → `model` → `cartridge` → `gateway` → `identity` → `soul` → `node`.

```bash
cargo publish -p tempo-x402
cargo publish -p tempo-x402-model
cargo publish -p tempo-x402-cartridge
cargo publish -p tempo-x402-gateway
cargo publish -p tempo-x402-identity
cargo publish -p tempo-x402-soul
cargo publish -p tempo-x402-node
```

Then: `gh release create v{VERSION} --title "v{VERSION} — Title" --notes "..."`

## Docs Maintenance

- Each crate has a `CLAUDE.md` — structural, not detailed
- Update when: dependencies change, public API changes, cross-crate impacts change
- New crate → must have CLAUDE.md (security-audit CI verifies)
