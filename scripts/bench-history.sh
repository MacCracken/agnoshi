#!/bin/sh
# bench-history.sh -- run Cyrius benchmarks, append to CSV history
#
# Usage:
#   sh scripts/bench-history.sh                 # defaults to bench-history.csv
#   sh scripts/bench-history.sh results.csv     # custom output file

set -eu

HISTORY_FILE="${1:-bench-history.csv}"
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
COMMIT=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")
BRANCH=$(git branch --show-current 2>/dev/null || echo "unknown")

# Create header if file doesn't exist
if [ ! -f "$HISTORY_FILE" ]; then
    echo "timestamp,commit,branch,benchmark,estimate_ns" > "$HISTORY_FILE"
fi

echo "=== agnoshi benchmark suite ==="
echo "  commit: $COMMIT"
echo "  branch: $BRANCH"
echo "  date:   $TIMESTAMP"
echo ""

# Build and run the benchmark binary
mkdir -p build
cyrius build tests/bench_core.bcyr build/bench_core

# Capture output
BENCH_OUTPUT=$(./build/bench_core 2>&1)
echo "$BENCH_OUTPUT"
echo ""

# Parse lines like: "  parse/list_files: 1us avg (min=1us max=8us) [10000 iters]"
# Extract benchmark name and avg time in ns
echo "$BENCH_OUTPUT" | while IFS= read -r line; do
    # Skip non-benchmark lines
    case "$line" in
        *" avg "*)  ;;
        *) continue ;;
    esac

    # Extract name (before colon)
    NAME=$(echo "$line" | sed -E 's/^[[:space:]]*([^:]+):.*/\1/')

    # Extract avg value and unit (e.g. "1us", "680ns")
    AVG=$(echo "$line" | sed -E 's/.*: ([0-9]+)([a-z]+) avg.*/\1 \2/')
    VAL=$(echo "$AVG" | awk '{print $1}')
    UNIT=$(echo "$AVG" | awk '{print $2}')

    # Normalize to nanoseconds
    case "$UNIT" in
        ns)  NS="$VAL" ;;
        us)  NS=$((VAL * 1000)) ;;
        ms)  NS=$((VAL * 1000000)) ;;
        s)   NS=$((VAL * 1000000000)) ;;
        *)   continue ;;
    esac

    echo "$TIMESTAMP,$COMMIT,$BRANCH,$NAME,$NS" >> "$HISTORY_FILE"
done

echo "Results appended to $HISTORY_FILE"
