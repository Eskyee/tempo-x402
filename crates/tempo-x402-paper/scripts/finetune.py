#!/usr/bin/env python3
"""
Self-play LoRA fine-tuning script for the paper.

Takes JSONL training data (verified solutions from self-play loop)
and fine-tunes a Qwen model using LoRA.

Usage:
  python finetune.py \
    --base-model Qwen/Qwen2.5-Coder-0.5B-Instruct \
    --data selfplay_runs/training_data/train.jsonl \
    --output checkpoints/iter_1 \
    --lora-rank 16 \
    --epochs 3 \
    --lr 2e-4

After fine-tuning, export to GGUF for llama.cpp inference:
  python export_gguf.py --checkpoint checkpoints/iter_1 --output models/qwen-iter-1.gguf
"""

import argparse
import json
import os
import sys

def load_training_data(path):
    """Load JSONL training examples."""
    examples = []
    with open(path, "r", encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if line:
                examples.append(json.loads(line))
    return examples

def format_for_training(examples):
    """Format examples for instruction fine-tuning."""
    formatted = []
    for ex in examples:
        # Chat template format for instruction-tuned models
        text = (
            f"<|im_start|>system\nYou are an expert Rust programmer. "
            f"Write complete, working Rust code.<|im_end|>\n"
            f"<|im_start|>user\n{ex['instruction']}<|im_end|>\n"
            f"<|im_start|>assistant\n{ex['output']}<|im_end|>"
        )
        formatted.append({"text": text})
    return formatted

def main():
    parser = argparse.ArgumentParser(description="Self-play LoRA fine-tuning")
    parser.add_argument("--base-model", default="Qwen/Qwen2.5-Coder-0.5B-Instruct",
                        help="Base model name or path")
    parser.add_argument("--data", required=True, help="JSONL training data path")
    parser.add_argument("--output", required=True, help="Output directory for checkpoint")
    parser.add_argument("--lora-rank", type=int, default=16, help="LoRA rank")
    parser.add_argument("--lora-alpha", type=int, default=32, help="LoRA alpha")
    parser.add_argument("--epochs", type=int, default=3, help="Training epochs")
    parser.add_argument("--lr", type=float, default=2e-4, help="Learning rate")
    parser.add_argument("--batch-size", type=int, default=4, help="Batch size")
    parser.add_argument("--max-length", type=int, default=2048, help="Max sequence length")
    parser.add_argument("--gradient-accumulation", type=int, default=4,
                        help="Gradient accumulation steps")
    args = parser.parse_args()

    # Check dependencies
    try:
        import torch
        from transformers import AutoModelForCausalLM, AutoTokenizer, TrainingArguments, Trainer
        from peft import LoraConfig, get_peft_model, TaskType
        from datasets import Dataset
    except ImportError as e:
        print(f"Missing dependency: {e}")
        print("Install: pip install transformers peft datasets accelerate torch")
        sys.exit(1)

    # Load data
    print(f"Loading training data from {args.data}...")
    raw_examples = load_training_data(args.data)
    if not raw_examples:
        print("No training examples found!")
        sys.exit(1)

    # Deduplicate by slug (keep latest solution per problem)
    seen = {}
    for ex in raw_examples:
        seen[ex.get("slug", "")] = ex
    examples = list(seen.values())
    print(f"  {len(raw_examples)} raw → {len(examples)} deduplicated examples")

    formatted = format_for_training(examples)
    dataset = Dataset.from_list(formatted)

    # Load model + tokenizer
    print(f"Loading base model: {args.base_model}...")
    tokenizer = AutoTokenizer.from_pretrained(args.base_model, trust_remote_code=True)
    if tokenizer.pad_token is None:
        tokenizer.pad_token = tokenizer.eos_token

    model = AutoModelForCausalLM.from_pretrained(
        args.base_model,
        torch_dtype=torch.float16,
        device_map="auto",
        trust_remote_code=True,
    )

    # Apply LoRA
    print(f"Applying LoRA (rank={args.lora_rank}, alpha={args.lora_alpha})...")
    lora_config = LoraConfig(
        task_type=TaskType.CAUSAL_LM,
        r=args.lora_rank,
        lora_alpha=args.lora_alpha,
        lora_dropout=0.05,
        target_modules=["q_proj", "k_proj", "v_proj", "o_proj",
                         "gate_proj", "up_proj", "down_proj"],
    )
    model = get_peft_model(model, lora_config)
    trainable = sum(p.numel() for p in model.parameters() if p.requires_grad)
    total = sum(p.numel() for p in model.parameters())
    print(f"  Trainable: {trainable:,} / {total:,} ({100*trainable/total:.2f}%)")

    # Tokenize
    def tokenize(batch):
        return tokenizer(
            batch["text"],
            truncation=True,
            max_length=args.max_length,
            padding="max_length",
        )

    print("Tokenizing...")
    tokenized = dataset.map(tokenize, batched=True, remove_columns=["text"])
    tokenized = tokenized.map(lambda x: {"labels": x["input_ids"]})

    # Train
    os.makedirs(args.output, exist_ok=True)

    training_args = TrainingArguments(
        output_dir=args.output,
        num_train_epochs=args.epochs,
        per_device_train_batch_size=args.batch_size,
        gradient_accumulation_steps=args.gradient_accumulation,
        learning_rate=args.lr,
        fp16=True,
        logging_steps=10,
        save_strategy="epoch",
        warmup_ratio=0.1,
        lr_scheduler_type="cosine",
        report_to="none",
    )

    trainer = Trainer(
        model=model,
        args=training_args,
        train_dataset=tokenized,
    )

    print(f"\nStarting training: {args.epochs} epochs, {len(examples)} examples...")
    trainer.train()

    # Save LoRA adapter
    adapter_path = os.path.join(args.output, "adapter")
    model.save_pretrained(adapter_path)
    tokenizer.save_pretrained(adapter_path)
    print(f"\nLoRA adapter saved to {adapter_path}")

    # Save training metadata
    meta = {
        "base_model": args.base_model,
        "lora_rank": args.lora_rank,
        "lora_alpha": args.lora_alpha,
        "epochs": args.epochs,
        "lr": args.lr,
        "examples": len(examples),
        "trainable_params": trainable,
        "total_params": total,
    }
    with open(os.path.join(args.output, "training_meta.json"), "w") as f:
        json.dump(meta, f, indent=2)
    print(f"Training metadata saved to {args.output}/training_meta.json")

if __name__ == "__main__":
    main()
