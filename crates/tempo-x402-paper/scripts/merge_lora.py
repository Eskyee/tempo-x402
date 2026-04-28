#!/usr/bin/env python
"""Merge a PEFT LoRA adapter into its base model and save as HF-style safetensors.

The resulting directory can be loaded directly by candle (or any HF-compatible
loader) — it contains model.safetensors, config.json, tokenizer files.

Usage:
    python merge_lora.py --adapter <adapter_dir> [--base-model <hf-repo-or-path>] [--output <dir>]

Defaults:
    --base-model defaults to whatever is recorded in the adapter's adapter_config.json
    --output defaults to <adapter_dir>/../merged
"""
import argparse
import json
import os
import sys
from pathlib import Path


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--adapter", required=True, help="Path to PEFT adapter dir (contains adapter_config.json).")
    ap.add_argument("--base-model", default=None, help="Override base model (HF repo id or local path). Defaults to adapter config.")
    ap.add_argument("--output", default=None, help="Output dir. Defaults to <adapter>/../merged.")
    ap.add_argument("--dtype", default="bfloat16", choices=["bfloat16", "float16", "float32"])
    args = ap.parse_args()

    adapter_dir = Path(args.adapter).resolve()
    if not adapter_dir.exists():
        sys.exit(f"adapter dir not found: {adapter_dir}")

    # Resolve base model
    adapter_cfg_path = adapter_dir / "adapter_config.json"
    base_model = args.base_model
    if base_model is None:
        if not adapter_cfg_path.exists():
            sys.exit(f"no --base-model given and {adapter_cfg_path} missing")
        cfg = json.loads(adapter_cfg_path.read_text(encoding="utf-8"))
        base_model = cfg.get("base_model_name_or_path")
        if not base_model:
            sys.exit("could not resolve base model from adapter config")

    # Resolve output dir
    if args.output is None:
        output_dir = adapter_dir.parent / "merged"
    else:
        output_dir = Path(args.output).resolve()
    output_dir.mkdir(parents=True, exist_ok=True)

    print(f"[merge_lora] adapter   : {adapter_dir}")
    print(f"[merge_lora] base model: {base_model}")
    print(f"[merge_lora] output    : {output_dir}")
    print(f"[merge_lora] dtype     : {args.dtype}")

    # Import heavy libs only after arg parsing, so --help is fast.
    import torch
    from transformers import AutoModelForCausalLM, AutoTokenizer
    from peft import PeftModel

    dtype = {
        "bfloat16": torch.bfloat16,
        "float16": torch.float16,
        "float32": torch.float32,
    }[args.dtype]

    print("[merge_lora] loading base model ...")
    base = AutoModelForCausalLM.from_pretrained(
        base_model,
        torch_dtype=dtype,
        device_map="cpu",
    )

    print("[merge_lora] attaching LoRA adapter ...")
    peft = PeftModel.from_pretrained(base, str(adapter_dir))
    print("[merge_lora] merging and unloading ...")
    merged = peft.merge_and_unload()

    print("[merge_lora] saving merged weights ...")
    merged.save_pretrained(str(output_dir), safe_serialization=True)

    # Also copy the tokenizer — required for candle.
    print("[merge_lora] saving tokenizer ...")
    tok = AutoTokenizer.from_pretrained(base_model)
    tok.save_pretrained(str(output_dir))

    print(f"[merge_lora] done -> {output_dir}")


if __name__ == "__main__":
    main()
