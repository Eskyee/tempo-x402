#!/usr/bin/env python3
"""
Export a LoRA fine-tuned model to GGUF format for llama.cpp inference.

Merges LoRA adapter into the base model, then converts to GGUF.

Usage:
  python export_gguf.py \
    --base-model Qwen/Qwen2.5-Coder-0.5B-Instruct \
    --adapter checkpoints/iter_1/adapter \
    --output models/qwen-iter-1.gguf \
    --quantize q4_k_m
"""

import argparse
import os
import subprocess
import sys
import tempfile

def main():
    parser = argparse.ArgumentParser(description="Export LoRA model to GGUF")
    parser.add_argument("--base-model", default="Qwen/Qwen2.5-Coder-0.5B-Instruct")
    parser.add_argument("--adapter", required=True,
                        help="Path to LoRA adapter dir, or merged model dir with --no-lora")
    parser.add_argument("--output", required=True, help="Output GGUF file path")
    parser.add_argument("--quantize", default="q4_k_m",
                        help="Quantization type (q4_k_m, q8_0, f16)")
    parser.add_argument("--no-lora", action="store_true",
                        help="Skip LoRA merge — adapter path is already a full HF model")
    args = parser.parse_args()

    try:
        import torch
        from transformers import AutoModelForCausalLM, AutoTokenizer
        if not args.no_lora:
            from peft import PeftModel
    except ImportError as e:
        print(f"Missing dependency: {e}")
        sys.exit(1)

    if args.no_lora:
        # Full fine-tune: adapter path IS the merged model
        merged_dir = args.adapter
        print(f"Using pre-merged model at {merged_dir}")
    else:
        # Step 1: Merge LoRA into base model
        print(f"Loading base model: {args.base_model}...")
        base_model = AutoModelForCausalLM.from_pretrained(
            args.base_model,
            torch_dtype=torch.float16,
            device_map="cpu",
            trust_remote_code=True,
        )
        tokenizer = AutoTokenizer.from_pretrained(args.base_model, trust_remote_code=True)

        print(f"Loading LoRA adapter: {args.adapter}...")
        model = PeftModel.from_pretrained(base_model, args.adapter)

        print("Merging LoRA weights into base model...")
        merged = model.merge_and_unload()

        # Step 2: Save merged model to temp directory
        merged_dir = tempfile.mkdtemp(prefix="merged_model_")
        print(f"Saving merged model to {merged_dir}...")
        merged.save_pretrained(merged_dir, safe_serialization=True)
        tokenizer.save_pretrained(merged_dir)

    # Step 3: Convert to GGUF using llama.cpp's convert script
    llama_cpp_dir = os.environ.get("LLAMA_CPP_DIR", "C:/llama-cpp")
    convert_script = os.path.join(llama_cpp_dir, "convert_hf_to_gguf.py")

    if not os.path.exists(convert_script):
        print(f"llama.cpp convert script not found at {convert_script}")
        print(f"Set LLAMA_CPP_DIR env var to your llama.cpp directory")
        print(f"Merged model saved at {merged_dir} — convert manually:")
        print(f"  python {convert_script} {merged_dir} --outfile {args.output} --outtype {args.quantize}")
        sys.exit(1)

    os.makedirs(os.path.dirname(args.output) or ".", exist_ok=True)

    # Convert HF → GGUF
    gguf_f16 = args.output.replace(".gguf", "-f16.gguf")
    print(f"Converting to GGUF (f16)...")
    subprocess.run([
        sys.executable, convert_script,
        merged_dir,
        "--outfile", gguf_f16,
        "--outtype", "f16",
    ], check=True)

    # Quantize if requested
    if args.quantize != "f16":
        quantize_bin = os.path.join(llama_cpp_dir, "build", "bin", "Release", "llama-quantize.exe")
        if not os.path.exists(quantize_bin):
            quantize_bin = os.path.join(llama_cpp_dir, "build", "bin", "llama-quantize.exe")
        if not os.path.exists(quantize_bin):
            quantize_bin = os.path.join(llama_cpp_dir, "build", "bin", "llama-quantize")
        if os.path.exists(quantize_bin):
            print(f"Quantizing to {args.quantize}...")
            subprocess.run([quantize_bin, gguf_f16, args.output, args.quantize], check=True)
            os.remove(gguf_f16)
        else:
            print(f"Quantize binary not found. F16 model at: {gguf_f16}")
            print(f"Quantize manually: llama-quantize {gguf_f16} {args.output} {args.quantize}")
    else:
        os.rename(gguf_f16, args.output)

    print(f"\nGGUF model saved to {args.output}")
    print(f"Use with: llama-server -m {args.output} --port 8081 -c 8192")

    # Cleanup temp dir (don't delete if --no-lora, it's the user's model)
    if not args.no_lora:
        import shutil
        shutil.rmtree(merged_dir, ignore_errors=True)

if __name__ == "__main__":
    main()
