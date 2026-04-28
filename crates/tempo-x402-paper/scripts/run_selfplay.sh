#!/bin/bash
# Self-play fine-tuning loop — the full experiment.
#
# Usage:
#   ./run_selfplay.sh [iterations] [problems]
#
# Prerequisites:
#   - llama-server built and in /c/llama-cpp/build/bin/Release/
#   - Base Qwen GGUF model in models/
#   - Python with torch, transformers, peft installed
#   - paper-bench built (cargo build --release --bin paper-bench)

set -e

ITERATIONS=${1:-10}
PROBLEMS=${2:-0}  # 0 = all 201
BASE_MODEL="Qwen/Qwen2.5-Coder-0.5B-Instruct"
BASE_GGUF="models/qwen2.5-coder-0.5b-instruct-q4_k_m.gguf"
LLAMA_SERVER="/c/llama-cpp/build/bin/Release/llama-server.exe"
SERVER_PORT=8081
OUTPUT_DIR="selfplay_runs"
SCRIPTS_DIR="crates/tempo-x402-paper/scripts"

echo "=== Self-Play Fine-Tuning Experiment ==="
echo "Iterations: $ITERATIONS"
echo "Problems: $PROBLEMS (0=all)"
echo "Base model: $BASE_MODEL"
echo ""

mkdir -p "$OUTPUT_DIR/training_data" "$OUTPUT_DIR/checkpoints" "$OUTPUT_DIR/results" "$OUTPUT_DIR/models"

# Copy base model as iteration 0
cp "$BASE_GGUF" "$OUTPUT_DIR/models/iter_0.gguf"

for iter in $(seq 0 $((ITERATIONS - 1))); do
    echo ""
    echo "========================================"
    echo "  ITERATION $iter / $((ITERATIONS - 1))"
    echo "========================================"

    MODEL_GGUF="$OUTPUT_DIR/models/iter_${iter}.gguf"

    # 1. Start llama-server with current model
    echo "[1/4] Starting llama-server with iter_${iter} model..."
    pkill -f llama-server 2>/dev/null || true
    sleep 1
    $LLAMA_SERVER -m "$MODEL_GGUF" --port $SERVER_PORT -c 8192 --host 127.0.0.1 &>/dev/null &
    SERVER_PID=$!
    sleep 4

    # Check server health
    if ! curl -s "http://127.0.0.1:$SERVER_PORT/health" | grep -q "ok"; then
        echo "ERROR: llama-server failed to start"
        exit 1
    fi

    # 2. Run benchmark
    echo "[2/4] Running benchmark (iter $iter)..."
    RESULT_FILE="$OUTPUT_DIR/results/iter_${iter}.json"
    PROBLEMS_ARG=""
    if [ "$PROBLEMS" -gt 0 ]; then
        PROBLEMS_ARG="--problems $PROBLEMS"
    fi

    cargo run --release --bin paper-bench -- score-local \
        --server-url "http://127.0.0.1:$SERVER_PORT" \
        --name "qwen-0.5b-iter-${iter}" \
        $PROBLEMS_ARG \
        --output "$RESULT_FILE"

    # Extract pass count
    PASSED=$(python -c "import json; d=json.load(open('$RESULT_FILE',encoding='utf-8')); print(d['passed'])")
    TOTAL=$(python -c "import json; d=json.load(open('$RESULT_FILE',encoding='utf-8')); print(d['total_problems'])")
    echo "  Result: $PASSED / $TOTAL passed"

    # 3. Extract training data from passed solutions
    echo "[3/4] Extracting training data..."
    python -c "
import json

# Load results
with open('$RESULT_FILE', encoding='utf-8') as f:
    results = json.load(f)

# Load existing training data
train_path = '$OUTPUT_DIR/training_data/train.jsonl'
existing = {}
try:
    with open(train_path, 'r', encoding='utf-8') as f:
        for line in f:
            ex = json.loads(line)
            existing[ex.get('slug', '')] = ex
except FileNotFoundError:
    pass

# Add new passing solutions
new_count = 0
for r in results['results']:
    if r['passed'] and r['solution'].strip():
        slug = r['slug']
        if slug not in existing:
            existing[slug] = {
                'instruction': f\"Write a complete Rust library (src/lib.rs) for: {slug}\",
                'output': r['solution'],
                'slug': slug,
                'tier': r['tier'],
                'iteration': $iter,
                'was_retry': False,
            }
            new_count += 1

# Write accumulated training data
with open(train_path, 'w', encoding='utf-8') as f:
    for ex in existing.values():
        f.write(json.dumps(ex) + '\n')

print(f'  {new_count} new examples, {len(existing)} total')
"

    # Stop server before fine-tuning (free GPU memory)
    kill $SERVER_PID 2>/dev/null || true
    sleep 2

    # 4. Fine-tune (skip on last iteration — no next model needed)
    NEXT_ITER=$((iter + 1))
    if [ $NEXT_ITER -lt $ITERATIONS ]; then
        TRAIN_COUNT=$(wc -l < "$OUTPUT_DIR/training_data/train.jsonl")
        if [ "$TRAIN_COUNT" -lt 2 ]; then
            echo "[4/4] Skipping fine-tune — not enough training data ($TRAIN_COUNT examples)"
            cp "$BASE_GGUF" "$OUTPUT_DIR/models/iter_${NEXT_ITER}.gguf"
        else
            echo "[4/4] Fine-tuning on $TRAIN_COUNT examples..."
            python "$SCRIPTS_DIR/finetune.py" \
                --base-model "$BASE_MODEL" \
                --data "$OUTPUT_DIR/training_data/train.jsonl" \
                --output "$OUTPUT_DIR/checkpoints/iter_${NEXT_ITER}" \
                --lora-rank 16 \
                --epochs 3 \
                --lr 2e-4

            echo "  Exporting to GGUF..."
            python "$SCRIPTS_DIR/export_gguf.py" \
                --base-model "$BASE_MODEL" \
                --adapter "$OUTPUT_DIR/checkpoints/iter_${NEXT_ITER}/adapter" \
                --output "$OUTPUT_DIR/models/iter_${NEXT_ITER}.gguf" \
                --quantize q4_k_m
        fi
    fi
done

# Print final summary
echo ""
echo "=== SELF-PLAY CONVERGENCE ==="
echo ""
printf "%-12s %8s %8s %10s\n" "Iteration" "Passed" "Total" "Rate"
echo "----------------------------------------"
for iter in $(seq 0 $((ITERATIONS - 1))); do
    RESULT_FILE="$OUTPUT_DIR/results/iter_${iter}.json"
    if [ -f "$RESULT_FILE" ]; then
        python -c "
import json
with open('$RESULT_FILE', encoding='utf-8') as f:
    d = json.load(f)
print(f'iter_{$iter:<8} {d[\"passed\"]:>8} {d[\"total_problems\"]:>8} {d[\"raw_pass_rate\"]:>9.1f}%')
"
    fi
done
echo ""
echo "Training data: $OUTPUT_DIR/training_data/train.jsonl"
echo "Results: $OUTPUT_DIR/results/"
echo "Models: $OUTPUT_DIR/models/"
