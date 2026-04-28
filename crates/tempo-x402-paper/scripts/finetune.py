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
    """Format examples for instruction fine-tuning with loss masking.

    Returns text with instruction/response split so we can mask loss
    on instruction tokens (only train on the code output).
    """
    formatted = []
    for ex in examples:
        # Chat template format for instruction-tuned models
        # We split at the assistant marker so we can mask instruction tokens
        instruction_part = (
            f"<|im_start|>system\nYou are an expert Rust programmer. "
            f"Write complete, working Rust code.<|im_end|>\n"
            f"<|im_start|>user\n{ex['instruction']}<|im_end|>\n"
            f"<|im_start|>assistant\n"
        )
        response_part = f"{ex['output']}<|im_end|>"
        full_text = instruction_part + response_part
        formatted.append({
            "text": full_text,
            "instruction_length": len(instruction_part),  # for loss masking
        })
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
    parser.add_argument("--lora-dropout", type=float, default=0.1,
                        help="LoRA dropout (higher for small datasets)")
    parser.add_argument("--full-finetune", action="store_true",
                        help="Full fine-tune (no LoRA). Uses gradient checkpointing + bf16.")
    parser.add_argument("--load-in-4bit", action="store_true",
                        help="Load base model in 4-bit (for larger models like 3B)")
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
    print(f"  {len(raw_examples)} raw -> {len(examples)} deduplicated examples")

    formatted = format_for_training(examples)
    dataset = Dataset.from_list(formatted)

    # Load model + tokenizer
    print(f"Loading base model: {args.base_model}...")
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
        print("  Loading in 4-bit (nf4) for large model training...")
    elif args.full_finetune:
        load_kwargs["torch_dtype"] = torch.bfloat16
        load_kwargs["device_map"] = "auto"
    else:
        load_kwargs["torch_dtype"] = torch.float16
        load_kwargs["device_map"] = "auto"

    model = AutoModelForCausalLM.from_pretrained(args.base_model, **load_kwargs)

    if args.full_finetune:
        # Full fine-tune: no LoRA, use gradient checkpointing
        print("Full fine-tune mode: gradient checkpointing enabled, no LoRA")
        model.gradient_checkpointing_enable()
        trainable = sum(p.numel() for p in model.parameters() if p.requires_grad)
        total = sum(p.numel() for p in model.parameters())
        print(f"  Trainable: {trainable:,} / {total:,} (100%)")
    else:
        # Apply LoRA
        print(f"Applying LoRA (rank={args.lora_rank}, alpha={args.lora_alpha})...")
        lora_config = LoraConfig(
            task_type=TaskType.CAUSAL_LM,
            r=args.lora_rank,
            lora_alpha=args.lora_alpha,
            lora_dropout=args.lora_dropout,
            target_modules=["q_proj", "k_proj", "v_proj", "o_proj",
                             "gate_proj", "up_proj", "down_proj"],
        )
        model = get_peft_model(model, lora_config)
    trainable = sum(p.numel() for p in model.parameters() if p.requires_grad)
    total = sum(p.numel() for p in model.parameters())
    print(f"  Trainable: {trainable:,} / {total:,} ({100*trainable/total:.2f}%)")

    # Tokenize with loss masking — only compute loss on the response tokens
    def tokenize_with_masking(example):
        full_tokens = tokenizer(
            example["text"],
            truncation=True,
            max_length=args.max_length,
            padding="max_length",
        )
        # Tokenize just the instruction part to find where response starts
        instruction_text = example["text"][:example["instruction_length"]]
        instruction_tokens = tokenizer(
            instruction_text,
            truncation=True,
            max_length=args.max_length,
            add_special_tokens=False,
        )
        mask_len = len(instruction_tokens["input_ids"])

        # Create labels: -100 for instruction tokens (ignored in loss), real ids for response
        labels = list(full_tokens["input_ids"])
        for i in range(min(mask_len, len(labels))):
            labels[i] = -100
        # Also mask padding tokens
        for i in range(len(labels)):
            if full_tokens["attention_mask"][i] == 0:
                labels[i] = -100

        full_tokens["labels"] = labels
        return full_tokens

    print("Tokenizing with loss masking (instruction tokens masked)...")
    tokenized = dataset.map(
        tokenize_with_masking,
        remove_columns=["text", "instruction_length"],
    )

    # Train
    os.makedirs(args.output, exist_ok=True)

    use_bf16 = args.full_finetune or args.load_in_4bit
    training_args = TrainingArguments(
        output_dir=args.output,
        num_train_epochs=args.epochs,
        per_device_train_batch_size=args.batch_size,
        gradient_accumulation_steps=args.gradient_accumulation,
        learning_rate=args.lr,
        fp16=not use_bf16,
        bf16=use_bf16,
        logging_steps=10,
        save_strategy="epoch",
        warmup_ratio=0.1,
        lr_scheduler_type="cosine",
        report_to="none",
        gradient_checkpointing=args.full_finetune,
    )

    trainer = Trainer(
        model=model,
        args=training_args,
        train_dataset=tokenized,
    )

    print(f"\nStarting training: {args.epochs} epochs, {len(examples)} examples...")
    trainer.train()

    # Save model
    if args.full_finetune:
        # Full fine-tune: save the entire merged model
        merged_path = os.path.join(args.output, "merged")
        model.save_pretrained(merged_path, safe_serialization=True)
        tokenizer.save_pretrained(merged_path)
        print(f"\nFull model saved to {merged_path}")
    else:
        # LoRA: save adapter only
        adapter_path = os.path.join(args.output, "adapter")
        model.save_pretrained(adapter_path)
        tokenizer.save_pretrained(adapter_path)
        print(f"\nLoRA adapter saved to {adapter_path}")

    # Save training metadata
    meta = {
        "base_model": args.base_model,
        "full_finetune": args.full_finetune,
        "lora_rank": 0 if args.full_finetune else args.lora_rank,
        "lora_alpha": 0 if args.full_finetune else args.lora_alpha,
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
