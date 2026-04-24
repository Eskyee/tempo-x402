# Paper: The Rust Compiler is a Free Self-Play Verifier

Short paper (~4 pp) reporting that Qwen2.5-Coder-0.5B + compiler-verified self-play
lifts pass@1 on a 201-problem Rust benchmark from 1.5% to 16.4%, and pass@5 to
25.9% — a 17× relative gain with no humans in the training loop.

## Build the PDF

```bash
cd crates/tempo-x402-paper/paper
pdflatex main.tex
bibtex main
pdflatex main.tex
pdflatex main.tex
```

Or with `latexmk`:

```bash
latexmk -pdf main.tex
```

## Regenerate figures

```bash
python ../scripts/plot_figures.py
```

Writes `figures/selfplay_curve.pdf` and `figures/per_tier.pdf` from the benchmark
result JSONs at `selfplay_runs_v2/...`.

## Reproduce the numbers (GPU, candle-native)

```bash
# (1) From repo root — build the benchmark + candle CUDA kernels.
#     On Windows this must run inside the VS Developer environment so nvcc
#     can find cl.exe. The scripts/build_candle.bat wrapper handles that.
scripts\build_candle.bat
# On Linux / no VS wrapper needed:
cargo build --release -p tempo-x402-paper --bin paper-bench

# (2) Score the base 0.5B model (HF cache or any HF-style directory).
paper-bench score-candle \
    --model-dir ~/.cache/huggingface/hub/models--Qwen--Qwen2.5-Coder-0.5B-Instruct/snapshots/<sha>/ \
    --name qwen-0.5b-base \
    --output selfplay_runs_v2/results/qwen-0.5b-base-gpu.json

# (3) Score the full-SFT fine-tune (already merged to full safetensors).
paper-bench score-candle \
    --model-dir selfplay_runs_v2/checkpoints/full_ft/merged \
    --name qwen-0.5b-full-ft \
    --output selfplay_runs_v2/results/qwen-0.5b-full-ft-gpu.json

# (4) pass@5 variant.
paper-bench score-candle \
    --model-dir selfplay_runs_v2/checkpoints/full_ft/merged \
    --name qwen-0.5b-full-ft-pass5 \
    --samples 5 --temperature 0.7 \
    --output selfplay_runs_v2/results/qwen-0.5b-full-ft-pass5-gpu.json
```

LoRA ablations (r8, r32) require merging their adapter first:

```bash
python crates/tempo-x402-paper/scripts/merge_lora.py \
    --adapter selfplay_runs_v2/checkpoints/rank32/adapter \
    --output selfplay_runs_v2/checkpoints/rank32/merged
paper-bench score-candle --model-dir selfplay_runs_v2/checkpoints/rank32/merged ...
```

Or merge everything under `checkpoints/` in one shot:

```bash
python crates/tempo-x402-paper/scripts/merge_all_lora.py
```

## Status

- [x] 4-page focused draft (Option A: 0.5B-only headline)
- [x] Self-play convergence figure
- [x] Per-tier bar chart
- [x] Failure-mode table (compile-fail → test-fail migration)
- [x] Candle/CUDA inference path (replaces llama.cpp)
- [x] `scripts/build_candle.bat` — Windows VS-env build wrapper
- [x] `scripts/merge_lora.py` / `merge_all_lora.py`
- [ ] Rerun main table on GPU via candle (blocked on CUDA build finishing)
- [ ] Verify results match Q4/llama-server numbers within tolerance
- [ ] Author/affiliation block finalized
- [ ] Arxiv formatting pass

## Result-file index

From repo root:

| Row | File |
|-----|------|
| 0.5B baseline (llama.cpp Q4, archival) | `selfplay_runs/results/iter_0.json` |
| 0.5B LoRA r8 | `selfplay_runs_v2/iter_4_eval/results/` |
| 0.5B LoRA r32 | `selfplay_runs_v2/rank32_eval/results/` |
| 0.5B Full SFT | `selfplay_runs_v2/full_ft_eval/results/iteration_0.json` |
| 0.5B Full SFT pass@5 | `selfplay_runs_v2/results/full_ft_pass5.json` |
| 0.5B candle GPU reruns | `selfplay_runs_v2/results/qwen-0.5b-*-gpu.json` (pending) |
