#!/bin/sh
# test.sh -- run all agnoshi tests
set -e

mkdir -p build

echo "=== Building agnsh ==="
cyrius build src/agnsh.cyr build/agnsh
echo "Binary: $(wc -c < build/agnsh) bytes"
echo ""

echo "=== Running core unit tests ==="
cyrius build tests/test_core.tcyr build/test_core
./build/test_core
echo ""

echo "=== Running security unit tests ==="
cyrius build tests/test_security.tcyr build/test_security
./build/test_security
echo ""

echo "=== Running smoke test ==="
sh scripts/smoke-test.sh build/agnsh
echo ""

echo "=== Running benchmarks ==="
cyrius build tests/bench_core.bcyr build/bench_core
./build/bench_core
echo ""

echo "=== All tests passed ==="
