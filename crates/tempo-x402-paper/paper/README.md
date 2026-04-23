# Paper: Compiler-Verified Self-Play Fine-Tuning of Small Rust Coding Models

## Build

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

## Status

- [x] Draft structure
- [x] Abstract, intro, related work
- [x] Benchmark description, method
- [x] Main results table with current numbers (0.5B + 3B)
- [ ] 3B DPO benchmark number (running)
- [ ] 3B pass@5 number (pending)
- [ ] FCM preview / future work section (pending)
- [ ] Self-play convergence figure
- [ ] Tier-stratified scaling figure
- [ ] Arxiv-ready formatting pass

## Figures

Placeholder PDFs go in `figures/`. Generate from JSON results with:

```bash
python ../scripts/plot_selfplay.py --results ../../../selfplay_runs_v2/results --output figures/
```

## Reproducing the numbers

Results file locations (from repo root):

| Row | File |
|-----|------|
| 0.5B baseline | `selfplay_runs/results/iter_0.json` |
| 0.5B LoRA r8 | `selfplay_runs_v2/results/iter_4_eval/` |
| 0.5B LoRA r32 | `selfplay_runs_v2/results/rank32_eval/` |
| 0.5B Full SFT | `selfplay_runs_v2/results/full_ft_eval/` |
| 0.5B Full SFT pass@5 | `selfplay_runs_v2/results/full_ft_pass5.json` |
| 3B baseline | `selfplay_runs_v2/results/qwen-3b-baseline.json` |
| 3B LoRA r16 (122) | `selfplay_runs_v2/results/qwen-3b-finetuned.json` |
| 3B LoRA r16 (178) | `selfplay_runs_v2/results/qwen-3b-sft-178.json` |
| 3B DPO | `selfplay_runs_v2/results/qwen-3b-dpo.json` (pending) |
