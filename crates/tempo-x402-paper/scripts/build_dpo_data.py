#!/usr/bin/env python
"""
Build DPO (Direct Preference Optimization) training data from benchmark results.

Novel idea: use cargo test pass/fail as preference signal.
For each problem where we have both a passing (Claude) solution and
failing (model) attempts, create a preference pair:
  chosen = passing solution
  rejected = failing attempt

This teaches the model "what good Rust code looks like" vs "what bad Rust code looks like"
without just memorizing the correct answer.

Usage:
  python build_dpo_data.py \
    --solutions selfplay_runs_v2/training_data/train.jsonl \
    --results selfplay_runs_v2/full_ft_eval/results/iteration_0.json \
    --output selfplay_runs_v2/training_data/dpo_pairs.jsonl
"""

import argparse
import json
import os
import sys


def load_solutions(path):
    """Load verified solutions (chosen responses)."""
    solutions = {}
    with open(path, "r", encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if line:
                ex = json.loads(line)
                solutions[ex["slug"]] = ex
    return solutions


def load_results(paths):
    """Load benchmark results (rejected responses from model attempts)."""
    all_results = {}
    for path in paths:
        if not os.path.exists(path):
            continue
        with open(path, "r", encoding="utf-8") as f:
            data = json.load(f)

        # Handle both list and dict formats
        items = data if isinstance(data, list) else [data]
        for item in items:
            for r in item.get("problem_results", item.get("results", [])):
                slug = r.get("slug", "")
                if not r.get("passed") and r.get("solution", "").strip():
                    # Keep longest failing solution per slug (most complete attempt)
                    if slug not in all_results or len(r["solution"]) > len(all_results[slug]):
                        all_results[slug] = r
    return all_results


def build_dpo_pairs(solutions, failed_results):
    """Build DPO preference pairs: chosen (passing) vs rejected (failing)."""
    pairs = []
    for slug, solution in solutions.items():
        if slug not in failed_results:
            continue

        failed = failed_results[slug]
        instruction = solution["instruction"]
        chosen = solution["output"]
        rejected = failed["solution"]

        # Skip if rejected is too short (empty/trivial) or identical to chosen
        if len(rejected.strip()) < 20:
            continue
        if rejected.strip() == chosen.strip():
            continue

        pairs.append({
            "instruction": instruction,
            "chosen": chosen,
            "rejected": rejected,
            "slug": slug,
            "tier": solution.get("tier", ""),
            "error_type": classify_error(failed.get("error", "")),
        })

    return pairs


def classify_error(error):
    """Classify the type of error for analysis."""
    if not error:
        return "unknown"
    if "error[E" in error or "could not compile" in error:
        return "compile_error"
    if "FAILED" in error or "panicked" in error:
        return "test_failure"
    if "timeout" in error:
        return "timeout"
    return "other"


def build_error_corrective_pairs(solutions, failed_results):
    """Build error-corrective training pairs: (broken_code + error) -> fixed_code.

    Novel: teaches the model to DEBUG, not just generate.
    """
    pairs = []
    for slug, solution in solutions.items():
        if slug not in failed_results:
            continue

        failed = failed_results[slug]
        error_msg = failed.get("error", "").strip()
        broken_code = failed["solution"].strip()
        fixed_code = solution["output"].strip()

        if len(broken_code) < 20 or not error_msg:
            continue
        if broken_code == fixed_code:
            continue

        # Instruction: fix this broken code given the error
        instruction = (
            f"The following Rust code for '{slug}' has errors. "
            f"Fix it so all tests pass.\n\n"
            f"Broken code:\n```rust\n{broken_code}\n```\n\n"
            f"Error:\n```\n{error_msg[:500]}\n```\n\n"
            f"Write the complete fixed src/lib.rs."
        )

        pairs.append({
            "instruction": instruction,
            "output": fixed_code,
            "slug": slug,
            "tier": solution.get("tier", ""),
            "error_type": classify_error(error_msg),
            "pair_type": "error_corrective",
        })

    return pairs


def main():
    parser = argparse.ArgumentParser(description="Build DPO training data")
    parser.add_argument("--solutions", required=True, help="JSONL of verified solutions")
    parser.add_argument("--results", nargs="+", required=True,
                        help="JSON benchmark result files (model's failed attempts)")
    parser.add_argument("--output", required=True, help="Output JSONL for DPO pairs")
    parser.add_argument("--error-output", default="",
                        help="Output JSONL for error-corrective pairs")
    args = parser.parse_args()

    print(f"Loading solutions from {args.solutions}...")
    solutions = load_solutions(args.solutions)
    print(f"  {len(solutions)} verified solutions")

    print(f"Loading failed results from {len(args.results)} files...")
    failed = load_results(args.results)
    print(f"  {len(failed)} unique failed attempts")

    # Build DPO pairs
    dpo_pairs = build_dpo_pairs(solutions, failed)
    print(f"\nDPO pairs: {len(dpo_pairs)}")

    from collections import Counter
    error_types = Counter(p["error_type"] for p in dpo_pairs)
    for et, c in error_types.most_common():
        print(f"  {et}: {c}")

    os.makedirs(os.path.dirname(args.output) or ".", exist_ok=True)
    with open(args.output, "w", encoding="utf-8") as f:
        for pair in dpo_pairs:
            f.write(json.dumps(pair) + "\n")
    print(f"DPO pairs saved to {args.output}")

    # Build error-corrective pairs
    if args.error_output:
        ec_pairs = build_error_corrective_pairs(solutions, failed)
        print(f"\nError-corrective pairs: {len(ec_pairs)}")
        with open(args.error_output, "w", encoding="utf-8") as f:
            for pair in ec_pairs:
                f.write(json.dumps(pair) + "\n")
        print(f"Error-corrective pairs saved to {args.error_output}")


if __name__ == "__main__":
    main()
