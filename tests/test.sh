#!/bin/sh
# test.sh -- run all agnoshi tests
set -e

mkdir -p build

echo "=== Building agnsh ==="
cyrius build src/agnsh.cyr build/agnsh
echo "Binary: $(wc -c < build/agnsh) bytes"
echo ""

# Every tests/test_*.tcyr suite, discovered the way CI discovers them — a new suite
# cannot be left out of the local run (test_parse_corpus was, until 1.9.16).
for t in tests/test_*.tcyr; do
    n=$(basename "$t" .tcyr)
    echo "=== Running $n ==="
    cyrius build "$t" "build/$n"
    "./build/$n"
    echo ""
done

echo "=== Running smoke test ==="
sh scripts/smoke-test.sh build/agnsh
echo ""

echo "=== Running benchmarks ==="
cyrius build tests/bench_core.bcyr build/bench_core
./build/bench_core
echo ""

echo "=== All tests passed ==="
