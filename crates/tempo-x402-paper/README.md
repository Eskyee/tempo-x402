# paper-bench — Self-Play Fine-Tuning Research

Research benchmark harness for the paper: *"Compiler-Verified Self-Play Fine-Tuning of a 0.5B Code Model"*

## Quick Start

```bash
# 1. Download Qwen model
python -c "from huggingface_hub import hf_hub_download; hf_hub_download('Qwen/Qwen2.5-Coder-0.5B-Instruct-GGUF', 'qwen2.5-coder-0.5b-instruct-q4_k_m.gguf', local_dir='models')"

# 2. Build llama.cpp and start server
cd /path/to/llama.cpp && cmake -B build -DCMAKE_BUILD_TYPE=Release && cmake --build build --config Release
./build/bin/Release/llama-server -m /path/to/models/qwen2.5-coder-0.5b-instruct-q4_k_m.gguf --port 8081 -c 8192

# 3. Build and run benchmark
cargo build --release --bin paper-bench
cargo run --release --bin paper-bench -- score-local --server-url http://127.0.0.1:8081 --name qwen-0.5b-base

# 4. Run full self-play experiment (10 iterations)
./crates/tempo-x402-paper/scripts/run_selfplay.sh 10

# 5. View results
cargo run --release --bin paper-bench -- summary
```

## Architecture

```
paper-bench CLI
  ├── score-claude    — Score Claude Opus (ceiling)
  ├── score-gemini    — Score Gemini Flash Lite (baseline)
  ├── score-local     — Score any GGUF model (Qwen, DeepSeek, etc.)
  ├── selfplay        — Run self-play iteration (generate → validate → accumulate)
  ├── fetch-humaneval — Download HumanEval-Rust from HuggingFace
  └── summary         — Compare all results

scripts/
  ├── finetune.py     — LoRA fine-tuning on verified solutions
  ├── export_gguf.py  — Convert fine-tuned model to GGUF
  └── run_selfplay.sh — Full experiment orchestrator
```

## The Self-Play Loop

```
Iteration 0: Base Qwen-0.5B → solve 201 problems → N pass
             Fine-tune on N verified solutions (LoRA)
Iteration 1: Fine-tuned model → solve 201 problems → N+M pass
             Fine-tune on N+M solutions
             ...
Iteration K: Model solves significantly more problems
```

The compiler (`cargo test`) is the ground truth oracle. Only code that passes tests enters the training set. No external API needed — the model improves through its own verified successes.

## Benchmarks

- **Opus-201**: 201 custom Rust problems across 6 tiers (embedded in `tempo-x402-soul`)
- **HumanEval-Rust**: Standard benchmark (164 problems from MultiPL-E)

## Four-Way Comparison (for the paper)

| Model | Role |
|-------|------|
| Claude Opus 4.6 | Ceiling — best available |
| Gemini Flash Lite | Baseline — current system |
| Qwen-0.5B base | Off-the-shelf, no fine-tuning |
| Qwen-0.5B + self-play | **THE CONTRIBUTION** |

## Requirements

- Rust toolchain (for `cargo test` validation)
- llama.cpp (for local model inference)
- Python 3.9+ with: torch, transformers, peft, datasets, accelerate
- ~2GB disk for model + checkpoints
- GPU recommended for fine-tuning (CPU works but slow)
