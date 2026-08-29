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

# Parse lines like:
#   "  parse/list_files: 2.429us avg (min=2.329us max=3.732us) [10000 iters]"
#   "  sanitize/basename: 78ns avg (min=75ns max=95ns) [10000 iters]"
#
# Averages may be DECIMAL. Cyrius 6.5.x's bench harness prints "2.429us";
# older harnesses printed a bare "2us". The previous pattern matched
# `([0-9]+)([a-z]+)` — integer only — so under 6.5.x every microsecond row
# failed to match, fell through `case "$UNIT"` to `*) continue`, and was
# dropped WITHOUT a word. A run recorded 4 of 10 benchmarks and still exited
# 0, quietly hollowing out the CSV this project treats as its proof. Parse
# the value as a float, scale to integer nanoseconds, and fail loud on any
# "avg" line that does not parse rather than skipping it.
PARSED=$(echo "$BENCH_OUTPUT" | awk \
    -v ts="$TIMESTAMP" -v commit="$COMMIT" -v branch="$BRANCH" '
    / avg / {
        if (match($0, /^[[:space:]]*[^:]+:/) == 0) { bad++; next }
        name = substr($0, RSTART, RLENGTH - 1)
        gsub(/^[[:space:]]+|[[:space:]]+$/, "", name)

        if (match($0, /:[[:space:]]*[0-9]+(\.[0-9]+)?(ns|us|ms|s) avg/) == 0) { bad++; next }
        field = substr($0, RSTART, RLENGTH)
        sub(/^:[[:space:]]*/, "", field)
        sub(/ avg$/, "", field)

        unit = field; gsub(/[0-9.]/, "", unit)
        val = field + 0
        if      (unit == "ns") ns = val
        else if (unit == "us") ns = val * 1000
        else if (unit == "ms") ns = val * 1000000
        else if (unit == "s")  ns = val * 1000000000
        else { bad++; next }

        printf "%s,%s,%s,%s,%.0f\n", ts, commit, branch, name, ns
        n++
    }
    END {
        if (bad > 0) {
            printf "bench-history: %d benchmark line(s) failed to parse\n", bad > "/dev/stderr"
            exit 1
        }
        if (n == 0) {
            print "bench-history: no benchmark lines parsed (harness output format changed?)" > "/dev/stderr"
            exit 1
        }
    }
')

echo "$PARSED" >> "$HISTORY_FILE"
echo "Recorded $(echo "$PARSED" | wc -l) benchmark(s)"

echo "Results appended to $HISTORY_FILE"
