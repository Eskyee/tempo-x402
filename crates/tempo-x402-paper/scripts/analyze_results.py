#!/usr/bin/env python3
"""Analyze benchmark results — failure modes, per-tier breakdown, convergence."""

import json
import sys
import os
import glob

def analyze_single(path):
    with open(path, encoding="utf-8") as f:
        data = json.load(f)

    model = data["model"]
    total = data["total_problems"]
    passed = data["passed"]
    raw = data["raw_pass_rate"]
    weighted = data["weighted_pass_rate"]

    # Per-tier
    tiers = {}
    for r in data["results"]:
        t = r["tier"]
        if t not in tiers:
            tiers[t] = {"passed": 0, "total": 0, "compile_err": 0, "test_fail": 0, "empty": 0}
        tiers[t]["total"] += 1
        if r["passed"]:
            tiers[t]["passed"] += 1
        elif not r["solution"].strip():
            tiers[t]["empty"] += 1
        elif "error[E" in r["error"]:
            tiers[t]["compile_err"] += 1
        elif "FAILED" in r["error"] or "panicked" in r["error"]:
            tiers[t]["test_fail"] += 1
        else:
            tiers[t]["compile_err"] += 1

    # Overall failure modes
    compile_err = sum(t["compile_err"] for t in tiers.values())
    test_fail = sum(t["test_fail"] for t in tiers.values())
    empty = sum(t["empty"] for t in tiers.values())

    print(f"\n{'='*60}")
    print(f"Model: {model}")
    print(f"Passed: {passed}/{total} ({raw:.1f}% raw, {weighted:.1f}% weighted)")
    print(f"\nFailure breakdown:")
    print(f"  Compile errors: {compile_err}")
    print(f"  Tests failed:   {test_fail}")
    print(f"  Empty/timeout:  {empty}")
    print(f"\nPer-tier:")
    for t in sorted(tiers):
        d = tiers[t]
        pct = 100 * d["passed"] / d["total"] if d["total"] > 0 else 0
        print(f"  {t}: {d['passed']}/{d['total']} ({pct:.0f}%) "
              f"[compile_err={d['compile_err']}, test_fail={d['test_fail']}, empty={d['empty']}]")

    return data

def convergence(result_dir):
    """Plot convergence across self-play iterations."""
    files = sorted(glob.glob(os.path.join(result_dir, "iter_*.json")))
    if not files:
        print(f"No iteration results found in {result_dir}")
        return

    print(f"\n{'='*60}")
    print("SELF-PLAY CONVERGENCE")
    print(f"{'='*60}")
    print(f"{'Iteration':<12} {'Passed':>8} {'Total':>8} {'Rate':>10} {'New':>8} {'Cumulative':>12}")
    print("-" * 62)

    cumulative_solved = set()
    for f in files:
        with open(f, encoding="utf-8") as fp:
            data = json.load(fp)
        iter_num = os.path.basename(f).replace("iter_", "").replace(".json", "")
        passed = data["passed"]
        total = data["total_problems"]
        rate = data["raw_pass_rate"]

        new_solved = set()
        for r in data["results"]:
            if r["passed"]:
                if r["slug"] not in cumulative_solved:
                    new_solved.add(r["slug"])
                cumulative_solved.add(r["slug"])

        print(f"iter_{iter_num:<8} {passed:>8} {total:>8} {rate:>9.1f}% {len(new_solved):>8} {len(cumulative_solved):>12}")

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python analyze_results.py <results.json> [results2.json ...]")
        print("       python analyze_results.py --convergence <selfplay_runs/results/>")
        sys.exit(1)

    if sys.argv[1] == "--convergence":
        convergence(sys.argv[2] if len(sys.argv) > 2 else "selfplay_runs/results")
    else:
        for path in sys.argv[1:]:
            analyze_single(path)
