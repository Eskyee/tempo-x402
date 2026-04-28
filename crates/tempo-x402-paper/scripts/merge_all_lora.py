#!/usr/bin/env python
"""Merge every LoRA checkpoint under selfplay_runs_v2/checkpoints/ into full safetensors dirs.

Outputs dirs at <checkpoint>/merged/ for each. Skips dirs that already have merged/.

Each merged dir is a drop-in HF-style directory that candle can load.
"""
import os
import subprocess
import sys
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parent.parent.parent.parent
CHECKPOINTS = REPO_ROOT / "selfplay_runs_v2" / "checkpoints"
MERGE_SCRIPT = Path(__file__).parent / "merge_lora.py"


def main():
    if not CHECKPOINTS.exists():
        sys.exit(f"no checkpoints dir: {CHECKPOINTS}")

    to_merge = []
    for child in sorted(CHECKPOINTS.iterdir()):
        if not child.is_dir():
            continue

        # Case 1: adapter/ subdir (LoRA style)
        adapter = child / "adapter"
        merged = child / "merged"
        if adapter.exists() and (adapter / "adapter_config.json").exists():
            if merged.exists():
                print(f"[skip] {child.name}/merged already exists")
            else:
                to_merge.append((adapter, merged))
            continue

        # Case 2: direct adapter in root (some PEFT layouts)
        if (child / "adapter_config.json").exists():
            if merged.exists():
                print(f"[skip] {child.name}/merged already exists")
            else:
                to_merge.append((child, child / "merged"))
            continue

        # Case 3: already merged full SFT (model.safetensors present at merged/)
        if merged.exists() and (merged / "model.safetensors").exists():
            print(f"[skip] {child.name}/merged already merged")
            continue

    if not to_merge:
        print("no LoRA adapters to merge")
        return

    print(f"\n{len(to_merge)} adapter(s) to merge:")
    for a, m in to_merge:
        print(f"  {a} -> {m}")
    print()

    failures = []
    for adapter, output in to_merge:
        print(f"\n=== merging {adapter} ===")
        result = subprocess.run(
            [sys.executable, str(MERGE_SCRIPT),
             "--adapter", str(adapter),
             "--output", str(output)],
            cwd=str(REPO_ROOT),
        )
        if result.returncode != 0:
            failures.append(adapter)
            print(f"[fail] {adapter}")
        else:
            print(f"[ok] {output}")

    if failures:
        print(f"\n{len(failures)} FAILED:")
        for f in failures:
            print(f"  {f}")
        sys.exit(1)
    else:
        print(f"\nall {len(to_merge)} adapter(s) merged successfully")


if __name__ == "__main__":
    main()
