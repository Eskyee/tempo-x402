#!/usr/bin/env python
"""Generate paper figures from benchmark result JSONs.

Usage: python plot_figures.py

Writes to crates/tempo-x402-paper/paper/figures/*.pdf.
"""
import json
import os
import sys
from collections import defaultdict

import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt

REPO_ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", ".."))
RESULTS_V2 = os.path.join(REPO_ROOT, "selfplay_runs_v2", "results")
ITERS = os.path.join(REPO_ROOT, "selfplay_runs_v2")
OUT = os.path.join(REPO_ROOT, "crates", "tempo-x402-paper", "paper", "figures")

os.makedirs(OUT, exist_ok=True)

plt.rcParams.update({
    "font.family": "serif",
    "font.size": 10,
    "axes.labelsize": 11,
    "axes.titlesize": 12,
    "legend.fontsize": 9,
    "figure.dpi": 150,
    "savefig.bbox": "tight",
    "savefig.pad_inches": 0.05,
})


def load(path):
    with open(path, "r", encoding="utf-8") as f:
        return json.load(f)


def selfplay_curve():
    """Self-play iteration curve: 0.5B LoRA r8 conservative."""
    points = []  # (iter, passed, train_examples)
    for i in range(0, 5):
        p = os.path.join(ITERS, f"iter_{i}_eval", "results", "iteration_0.json")
        if os.path.exists(p):
            d = load(p)
            points.append((
                i,
                d.get("problems_passed", 0),
                d.get("total_training_examples", 0),
            ))
    # Add iter_0 (baseline) from iter_0.json if iter_0_eval doesn't exist
    if not points or points[0][0] != 0:
        base = os.path.join(REPO_ROOT, "selfplay_runs", "results", "iter_0.json")
        if os.path.exists(base):
            d = load(base)
            points.insert(0, (0, d.get("problems_passed", 0), 0))

    if not points:
        print("No iteration data found")
        return

    xs = [p[0] for p in points]
    passed = [p[1] for p in points]
    train_ex = [p[2] for p in points]

    fig, ax1 = plt.subplots(figsize=(5.5, 3.2))
    ax1.plot(xs, passed, "o-", color="#1f77b4", linewidth=1.8, markersize=6,
             label="Problems passed (pass@1)")
    ax1.set_xlabel("Self-play iteration")
    ax1.set_ylabel("Problems passed (/ 201)", color="#1f77b4")
    ax1.tick_params(axis="y", labelcolor="#1f77b4")
    ax1.grid(True, alpha=0.25)
    ax1.set_xticks(xs)

    ax2 = ax1.twinx()
    ax2.bar(xs, train_ex, color="#ff7f0e", alpha=0.25, width=0.6,
            label="Cumulative training examples")
    ax2.set_ylabel("Cumulative training examples", color="#ff7f0e")
    ax2.tick_params(axis="y", labelcolor="#ff7f0e")

    fig.suptitle("0.5B self-play convergence (LoRA r8, conservative)", y=1.02)
    fig.tight_layout()
    out = os.path.join(OUT, "selfplay_curve.pdf")
    fig.savefig(out)
    plt.close(fig)
    print(f"wrote {out}")


def tier_stratified():
    """Per-tier pass counts across models."""
    # 0.5B-only bar chart for the main paper (keeps the paper focused).
    # The 3B configurations live in the extended technical report.
    configs = [
        ("Base",       os.path.join(REPO_ROOT, "selfplay_runs", "results", "iter_0.json")),
        ("Full SFT",   os.path.join(REPO_ROOT, "selfplay_runs_v2", "full_ft_eval", "results", "iteration_0.json")),
        ("Full SFT, pass@5", os.path.join(RESULTS_V2, "full_ft_pass5.json")),
    ]

    tiers = ["tier1", "tier2", "tier3", "tier4", "tier5", "tier6"]
    tier_totals = {"tier1": 108, "tier2": 25, "tier3": 25, "tier4": 23, "tier5": 10, "tier6": 10}

    data = {}  # config_name -> {tier: passed}
    for name, path in configs:
        if not os.path.exists(path):
            print(f"WARN missing {path}")
            continue
        d = load(path)
        # Selfplay loop dumps use `problem_results` + `difficulty`;
        # score-local dumps use `results` + `tier`. Handle both.
        results = d.get("results") or d.get("problem_results") or []
        by_tier = defaultdict(int)
        for r in results:
            t = r.get("tier") or r.get("difficulty") or r.get("problem_tier") or ""
            if r.get("passed", False):
                by_tier[t] += 1
        data[name] = dict(by_tier)

    if not data:
        print("No tier data")
        return

    fig, ax = plt.subplots(figsize=(7.0, 3.6))
    n_configs = len(data)
    bar_w = 0.8 / n_configs
    x = list(range(len(tiers)))

    colors = plt.get_cmap("tab10").colors
    for i, (name, tier_counts) in enumerate(data.items()):
        heights = [tier_counts.get(t, 0) for t in tiers]
        fractions = [h / tier_totals[t] for h, t in zip(heights, tiers)]
        offset = (i - n_configs / 2 + 0.5) * bar_w
        ax.bar([xi + offset for xi in x], fractions, bar_w,
               label=name, color=colors[i % len(colors)])

    ax.set_xticks(x)
    ax.set_xticklabels([f"T{i+1}\n({tier_totals[t]})" for i, t in enumerate(tiers)])
    ax.set_ylabel("Pass rate")
    ax.set_title("Per-tier pass rate across configurations")
    ax.set_ylim(0, 1.0)
    ax.yaxis.set_major_formatter(plt.FuncFormatter(lambda y, _: f"{int(y*100)}%"))
    ax.grid(True, axis="y", alpha=0.25)
    ax.legend(ncol=2, loc="upper right", framealpha=0.9)
    fig.tight_layout()
    out = os.path.join(OUT, "per_tier.pdf")
    fig.savefig(out)
    plt.close(fig)
    print(f"wrote {out}")


def main():
    selfplay_curve()
    tier_stratified()


if __name__ == "__main__":
    main()
