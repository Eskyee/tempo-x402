#!/usr/bin/env python
"""
DPO (Direct Preference Optimization) trainer for code generation.

Novel: uses compiler pass/fail as preference signal. The model learns to
prefer code that compiles and passes tests over code that doesn't.

This is applied AFTER SFT (supervised fine-tuning). The pipeline:
  1. SFT on verified solutions (finetune.py)
  2. DPO on pass/fail preference pairs (this script)

Usage:
  python train_dpo.py \
    --base-model selfplay_runs_v2/checkpoints/full_ft_178/merged \
    --dpo-data selfplay_runs_v2/training_data/dpo_pairs.jsonl \
    --output selfplay_runs_v2/checkpoints/dpo \
    --beta 0.1 --epochs 1 --lr 5e-7
"""

import argparse
import json
import os
import sys


def load_dpo_pairs(path):
    """Load DPO preference pairs."""
    pairs = []
    with open(path, "r", encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if line:
                pairs.append(json.loads(line))
    return pairs


def format_for_dpo(pairs, tokenizer, max_length=2048):
    """Format DPO pairs into the expected dataset format."""
    formatted = []
    for pair in pairs:
        prompt = (
            f"<|im_start|>system\nYou are an expert Rust programmer. "
            f"Write complete, working Rust code.<|im_end|>\n"
            f"<|im_start|>user\n{pair['instruction']}<|im_end|>\n"
            f"<|im_start|>assistant\n"
        )
        chosen = f"{pair['chosen']}<|im_end|>"
        rejected = f"{pair['rejected']}<|im_end|>"

        formatted.append({
            "prompt": prompt,
            "chosen": chosen,
            "rejected": rejected,
        })
    return formatted


def main():
    parser = argparse.ArgumentParser(description="DPO training with compiler feedback")
    parser.add_argument("--base-model", required=True,
                        help="Path to SFT'd model (HF format) or model name")
    parser.add_argument("--dpo-data", required=True, help="JSONL of DPO preference pairs")
    parser.add_argument("--output", required=True, help="Output directory")
    parser.add_argument("--beta", type=float, default=0.1, help="DPO beta (KL penalty)")
    parser.add_argument("--epochs", type=int, default=1, help="Training epochs")
    parser.add_argument("--lr", type=float, default=5e-7, help="Learning rate (very small for DPO)")
    parser.add_argument("--batch-size", type=int, default=1, help="Batch size")
    parser.add_argument("--gradient-accumulation", type=int, default=4, help="Gradient accumulation")
    parser.add_argument("--max-length", type=int, default=2048, help="Max sequence length")
    parser.add_argument("--lora-rank", type=int, default=16, help="LoRA rank (0 for full)")
    parser.add_argument("--load-in-4bit", action="store_true", help="Load in 4-bit")
    args = parser.parse_args()

    try:
        import torch
        from transformers import AutoModelForCausalLM, AutoTokenizer
        from datasets import Dataset
        from trl import DPOTrainer, DPOConfig
        from peft import LoraConfig, get_peft_model, TaskType
    except ImportError as e:
        print(f"Missing dependency: {e}")
        print("Install: pip install trl>=0.9.0 transformers peft datasets")
        sys.exit(1)

    # Load DPO pairs
    print(f"Loading DPO data from {args.dpo_data}...")
    raw_pairs = load_dpo_pairs(args.dpo_data)
    print(f"  {len(raw_pairs)} preference pairs")

    if not raw_pairs:
        print("No DPO pairs found!")
        sys.exit(1)

    # Load model + tokenizer
    print(f"Loading model: {args.base_model}...")
    tokenizer = AutoTokenizer.from_pretrained(args.base_model, trust_remote_code=True)
    if tokenizer.pad_token is None:
        tokenizer.pad_token = tokenizer.eos_token

    load_kwargs = dict(trust_remote_code=True)
    if args.load_in_4bit:
        from transformers import BitsAndBytesConfig
        load_kwargs["quantization_config"] = BitsAndBytesConfig(
            load_in_4bit=True,
            bnb_4bit_compute_dtype=torch.bfloat16,
            bnb_4bit_quant_type="nf4",
        )
        load_kwargs["torch_dtype"] = torch.bfloat16
    else:
        load_kwargs["torch_dtype"] = torch.bfloat16
        load_kwargs["device_map"] = "auto"

    model = AutoModelForCausalLM.from_pretrained(args.base_model, **load_kwargs)

    # Apply LoRA if requested
    if args.lora_rank > 0:
        print(f"Applying LoRA (rank={args.lora_rank})...")
        lora_config = LoraConfig(
            task_type=TaskType.CAUSAL_LM,
            r=args.lora_rank,
            lora_alpha=args.lora_rank * 2,
            lora_dropout=0.05,
            target_modules=["q_proj", "k_proj", "v_proj", "o_proj",
                             "gate_proj", "up_proj", "down_proj"],
        )
        model = get_peft_model(model, lora_config)

    # Format dataset
    formatted = format_for_dpo(raw_pairs, tokenizer, args.max_length)
    dataset = Dataset.from_list(formatted)
    print(f"  Dataset: {len(dataset)} examples")

    # DPO training config
    os.makedirs(args.output, exist_ok=True)

    training_args = DPOConfig(
        output_dir=args.output,
        num_train_epochs=args.epochs,
        per_device_train_batch_size=args.batch_size,
        gradient_accumulation_steps=args.gradient_accumulation,
        learning_rate=args.lr,
        beta=args.beta,
        bf16=True,
        logging_steps=5,
        save_strategy="epoch",
        warmup_ratio=0.1,
        lr_scheduler_type="cosine",
        report_to="none",
        max_length=args.max_length,
        max_prompt_length=args.max_length // 2,
        remove_unused_columns=False,
    )

    # DPO Trainer
    trainer = DPOTrainer(
        model=model,
        args=training_args,
        train_dataset=dataset,
        processing_class=tokenizer,
    )

    print(f"\nStarting DPO training: beta={args.beta}, lr={args.lr}, {len(dataset)} pairs...")
    trainer.train()

    # Save
    save_path = os.path.join(args.output, "model")
    model.save_pretrained(save_path)
    tokenizer.save_pretrained(save_path)
    print(f"\nDPO model saved to {save_path}")

    # Save metadata
    meta = {
        "base_model": args.base_model,
        "dpo_beta": args.beta,
        "epochs": args.epochs,
        "lr": args.lr,
        "pairs": len(raw_pairs),
        "lora_rank": args.lora_rank,
    }
    with open(os.path.join(args.output, "dpo_meta.json"), "w") as f:
        json.dump(meta, f, indent=2)


if __name__ == "__main__":
    main()
