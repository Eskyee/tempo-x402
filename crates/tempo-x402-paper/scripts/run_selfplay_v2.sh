#!/bin/bash
# Self-play fine-tuning loop v2 — conservative LoRA, proper training data.
#
# Changes from v1:
#   - Uses paper-bench selfplay (generates full instructions with tests)
#   - Conservative LoRA (rank 8, alpha 16, 2 epochs) to avoid catastrophic forgetting
#   - Merges training data across iterations (deduplicates by slug, keeps best)
#   - Higher temperature (0.3) for diversity in solutions
#
# Usage:
#   ./run_selfplay_v2.sh [iterations] [output_dir]
#
# Prerequisites:
#   - llama-server built at /c/llama-cpp/build/bin/Release/
#   - Base Qwen GGUF at models/qwen2.5-coder-0.5b-instruct-q4_k_m.gguf
#   - Python with torch, transformers, peft, datasets installed
#   - paper-bench built: cargo build --release --bin paper-bench

set -e

ITERATIONS=${1:-5}
OUTPUT_DIR=${2:-selfplay_runs_v2}
BASE_MODEL="Qwen/Qwen2.5-Coder-0.5B-Instruct"
BASE_GGUF="models/qwen2.5-coder-0.5b-instruct-q4_k_m.gguf"
LLAMA_SERVER="/c/llama-cpp/build/bin/Release/llama-server.exe"
SERVER_PORT=8081
SCRIPTS_DIR="crates/tempo-x402-paper/scripts"

# Conservative LoRA params (prevent catastrophic forgetting on tiny datasets)
LORA_RANK=8
LORA_ALPHA=16
EPOCHS=2
LR="1e-4"

echo "=== Self-Play Fine-Tuning v2 ==="
echo "Iterations: $ITERATIONS"
echo "Output: $OUTPUT_DIR"
echo "LoRA: rank=$LORA_RANK, alpha=$LORA_ALPHA, epochs=$EPOCHS, lr=$LR"
echo ""

mkdir -p "$OUTPUT_DIR/training_data" "$OUTPUT_DIR/checkpoints" "$OUTPUT_DIR/results" "$OUTPUT_DIR/models"

# Copy base model as iteration 0
if [ ! -f "$OUTPUT_DIR/models/iter_0.gguf" ]; then
    cp "$BASE_GGUF" "$OUTPUT_DIR/models/iter_0.gguf"
fi

for iter in $(seq 0 $((ITERATIONS - 1))); do
    echo ""
    echo "========================================"
    echo "  ITERATION $iter / $((ITERATIONS - 1))"
    echo "========================================"

    MODEL_GGUF="$OUTPUT_DIR/models/iter_${iter}.gguf"
    ITER_DIR="$OUTPUT_DIR/iter_${iter}_run"

    # 1. Start llama-server with current model
    echo "[1/4] Starting llama-server with iter_${iter} model..."
    taskkill //F //IM llama-server.exe 2>/dev/null || true
    sleep 2
    $LLAMA_SERVER -m "$MODEL_GGUF" --port $SERVER_PORT -c 8192 --host 127.0.0.1 &>/dev/null &
    SERVER_PID=$!
    sleep 5

    # Check server health
    if ! curl -s "http://127.0.0.1:$SERVER_PORT/health" | grep -q "ok"; then
        echo "ERROR: llama-server failed to start"
        exit 1
    fi
    echo "  Server started (PID=$SERVER_PID)"

    # 2. Run selfplay iteration (uses paper-bench which generates proper training data)
    echo "[2/4] Running selfplay on all 201 problems..."
    cargo run --release --bin paper-bench -- selfplay \
        --server-url "http://127.0.0.1:$SERVER_PORT" \
        --iterations 1 \
        --problems 0 \
        --output-dir "$ITER_DIR" 2>&1 | tee "$OUTPUT_DIR/results/iter_${iter}_log.txt"

    # 3. Merge training data (deduplicate by slug, keep latest)
    echo "[3/4] Merging training data..."
    python -c "
import json, os

# Load existing accumulated training data
accumulated = {}
acc_path = '$OUTPUT_DIR/training_data/train.jsonl'
if os.path.exists(acc_path):
    with open(acc_path, 'r', encoding='utf-8') as f:
        for line in f:
            line = line.strip()
            if line:
                ex = json.loads(line)
                accumulated[ex.get('slug', '')] = ex

# Load new training data from this iteration
new_path = '$ITER_DIR/training_data/train.jsonl'
new_count = 0
if os.path.exists(new_path):
    with open(new_path, 'r', encoding='utf-8') as f:
        for line in f:
            line = line.strip()
            if line:
                ex = json.loads(line)
                slug = ex.get('slug', '')
                if slug not in accumulated:
                    new_count += 1
                accumulated[slug] = ex

# Write merged
with open(acc_path, 'w', encoding='utf-8') as f:
    for ex in accumulated.values():
        f.write(json.dumps(ex) + '\n')

total = len(accumulated)
print(f'  {new_count} new examples, {total} total accumulated')
"

    # Stop server before fine-tuning (free memory)
    taskkill //F //IM llama-server.exe 2>/dev/null || true
    sleep 2

    # 4. Fine-tune for next iteration
    NEXT_ITER=$((iter + 1))
    if [ $NEXT_ITER -lt $ITERATIONS ]; then
        TRAIN_COUNT=$(wc -l < "$OUTPUT_DIR/training_data/train.jsonl" 2>/dev/null || echo "0")
        if [ "$TRAIN_COUNT" -lt 3 ]; then
            echo "[4/4] Skipping fine-tune -- not enough data ($TRAIN_COUNT examples)"
            cp "$BASE_GGUF" "$OUTPUT_DIR/models/iter_${NEXT_ITER}.gguf"
        else
            echo "[4/4] Fine-tuning on $TRAIN_COUNT examples (LoRA rank=$LORA_RANK)..."
            python "$SCRIPTS_DIR/finetune.py" \
                --base-model "$BASE_MODEL" \
                --data "$OUTPUT_DIR/training_data/train.jsonl" \
                --output "$OUTPUT_DIR/checkpoints/iter_${NEXT_ITER}" \
                --lora-rank $LORA_RANK \
                --lora-alpha $LORA_ALPHA \
                --epochs $EPOCHS \
                --lr $LR

            echo "  Exporting to GGUF..."
            python "$SCRIPTS_DIR/export_gguf.py" \
                --base-model "$BASE_MODEL" \
                --adapter "$OUTPUT_DIR/checkpoints/iter_${NEXT_ITER}/adapter" \
                --output "$OUTPUT_DIR/models/iter_${NEXT_ITER}.gguf" \
                --quantize q4_k_m

            echo "  Model saved: $OUTPUT_DIR/models/iter_${NEXT_ITER}.gguf"
        fi
    fi

    # Save iteration summary
    python -c "
import json, os

result_file = '$ITER_DIR/results/iteration_0.json'
if os.path.exists(result_file):
    with open(result_file, 'r', encoding='utf-8') as f:
        data = json.load(f)
    passed = data.get('problems_passed', 0)
    total = data.get('problems_attempted', 0)
    rate = data.get('pass_rate', 0)
    examples = data.get('total_training_examples', 0)
    print(f'  iter {$iter}: {passed}/{total} passed ({rate:.1f}%), {examples} training examples')
else:
    print(f'  iter {$iter}: no results found')
"
done

# Print final convergence curve
echo ""
echo "=== SELF-PLAY CONVERGENCE (v2) ==="
echo ""
printf "%-12s %8s %8s %10s %12s\n" "Iteration" "Passed" "Total" "Rate" "Train Exs"
echo "---------------------------------------------------"
for iter in $(seq 0 $((ITERATIONS - 1))); do
    ITER_DIR="$OUTPUT_DIR/iter_${iter}_run"
    RESULT_FILE="$ITER_DIR/results/iteration_0.json"
    if [ -f "$RESULT_FILE" ]; then
        python -c "
import json
with open('$RESULT_FILE', 'r', encoding='utf-8') as f:
    d = json.load(f)
print(f'iter_{$iter:<8} {d[\"problems_passed\"]:>8} {d[\"problems_attempted\"]:>8} {d[\"pass_rate\"]:>9.1f}% {d[\"total_training_examples\"]:>12}')
"
    fi
done

TOTAL_EXAMPLES=$(wc -l < "$OUTPUT_DIR/training_data/train.jsonl" 2>/dev/null || echo "0")
echo ""
echo "Total accumulated training examples: $TOTAL_EXAMPLES"
echo "Training data: $OUTPUT_DIR/training_data/train.jsonl"
echo "Models: $OUTPUT_DIR/models/"
