#!/bin/bash

# Comprehensive Test Runner for WASM Core

set -e

echo "?? WASM Core Test Suite"
echo "??????????????????????????????????????????"
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Track results
TESTS_PASSED=0
TESTS_FAILED=0

# Function to run a test
run_test() {
    local test_name="$1"
    local test_command="$2"
    
    echo "?? Running: $test_name"
    if eval "$test_command" > /dev/null 2>&1; then
        echo -e "${GREEN}? PASSED${NC}: $test_name"
        ((TESTS_PASSED++))
    else
        echo -e "${RED}? FAILED${NC}: $test_name"
        ((TESTS_FAILED++))
    fi
    echo ""
}

# Cleanup function
cleanup() {
    echo "?? Cleaning up..."
    pkill -f "wasm-core" || true
    rm -f /tmp/wasm-core.sock
    rm -f metadata.json
}

# Set trap to cleanup on exit
trap cleanup EXIT

echo "??????????????????????????????????????????"
echo "?? STEP 1: Build All Components"
echo "??????????????????????????????????????????"
echo ""

# Build plugins
echo "Building plugins..."
./build-all-plugins.sh
echo ""

# Build core and client
echo "Building core and client..."
cargo build --release -p wasm-core
cargo build --release -p wasm-client
echo ""

echo "??????????????????????????????????????????"
echo "?? STEP 2: Start Core Server"
echo "??????????????????????????????????????????"
echo ""

# Start server in background
cargo run --release -p wasm-core &
SERVER_PID=$!
echo "Server PID: $SERVER_PID"

# Wait for server to start
echo "Waiting for server to start..."
sleep 3

# Check if server is running
if ! ps -p $SERVER_PID > /dev/null; then
    echo -e "${RED}? Server failed to start${NC}"
    exit 1
fi

if [ ! -S /tmp/wasm-core.sock ]; then
    echo -e "${RED}? Socket file not created${NC}"
    kill $SERVER_PID
    exit 1
fi

echo -e "${GREEN}? Server started successfully${NC}"
echo ""

echo "??????????????????????????????????????????"
echo "?? STEP 3: Run Test Scenarios"
echo "??????????????????????????????????????????"
echo ""

CLIENT="cargo run --release -p wasm-client --"

# Test 1: Load binaries
echo "??? Test 1: Load All Binaries ???"
run_test "Load reverser.wasm" "$CLIENT load --path ./plugins/reverser.wasm"
run_test "Load uppercase.wasm" "$CLIENT load --path ./plugins/uppercase.wasm"
run_test "Load counter.wasm" "$CLIENT load --path ./plugins/counter.wasm"
run_test "Load rot13.wasm" "$CLIENT load --path ./plugins/rot13.wasm"
run_test "Load env-reader.wasm" "$CLIENT load --path ./plugins/env-reader.wasm"

# Get binary IDs (for later tests)
REVERSER_ID=$($CLIENT load --path ./plugins/reverser.wasm 2>/dev/null | grep "Binary ID:" | awk '{print $3}')
UPPERCASE_ID=$($CLIENT load --path ./plugins/uppercase.wasm 2>/dev/null | grep "Binary ID:" | awk '{print $3}')
COUNTER_ID=$($CLIENT load --path ./plugins/counter.wasm 2>/dev/null | grep "Binary ID:" | awk '{print $3}')
ROT13_ID=$($CLIENT load --path ./plugins/rot13.wasm 2>/dev/null | grep "Binary ID:" | awk '{print $3}')
ENV_READER=$($CLIENT load --path ./plugins/env-reader.wasm 2>/dev/null | grep "Binary ID:" | awk '{print $3}')

# Test 2: List binaries
echo "??? Test 2: List Binaries ???"
run_test "List all binaries" "$CLIENT list"

# Test 3: Execute individual binaries
echo "??? Test 3: Execute Individual Binaries ???"
run_test "Execute reverser" "$CLIENT execute --binary-id $REVERSER_ID --input 'hello'"
run_test "Execute uppercase" "$CLIENT execute --binary-id $UPPERCASE_ID --input 'hello'"
run_test "Execute counter" "$CLIENT execute --binary-id $COUNTER_ID --input 'Hello 123'"
run_test "Execute rot13" "$CLIENT execute --binary-id $ROT13_ID --input 'secret'"
run_test "Execute env-reader" "$CLIENT execute --binary-id $ENV_READER --input 'test'"

# Test 4: Chain execution
echo "??? Test 4: Chain Execution ???"
run_test "Chain: uppercase ? reverser" "$CLIENT chain --binary-ids $UPPERCASE_ID,$REVERSER_ID --input 'hello'"
run_test "Chain: rot13 ? uppercase" "$CLIENT chain --binary-ids $ROT13_ID,$UPPERCASE_ID --input 'test'"
run_test "Chain: 3 plugins" "$CLIENT chain --binary-ids $UPPERCASE_ID,$ROT13_ID,$REVERSER_ID --input 'test'"

# Test 5: Different inputs
echo "??? Test 5: Different Input Types ???"
run_test "Empty string" "$CLIENT execute --binary-id $REVERSER_ID --input ''"
run_test "Long string" "$CLIENT execute --binary-id $REVERSER_ID --input 'This is a very long string to test the plugin'"
run_test "Numbers" "$CLIENT execute --binary-id $COUNTER_ID --input '12345'"
run_test "Special characters" "$CLIENT execute --binary-id $REVERSER_ID --input '!@#$%'"

# Test 6: Performance
echo "??? Test 6: Performance ???"
echo "Running 10 rapid executions..."
START=$(date +%s%N)
for i in {1..10}; do
    $CLIENT execute --binary-id $REVERSER_ID --input "test$i" > /dev/null 2>&1
done
END=$(date +%s%N)
DURATION=$(( (END - START) / 1000000 ))
echo "? Completed 10 executions in ${DURATION}ms (avg: $((DURATION / 10))ms)"
echo ""

# Test 7: Timeout handling
echo "??? Test 7: Timeout Handling ???"
run_test "Execute with timeout" "$CLIENT execute --binary-id $REVERSER_ID --input 'test' --timeout 5000"

# Test 8: Memory limits
echo "??? Test 8: Memory Limits ???"
run_test "Execute with memory limit" "$CLIENT execute --binary-id $REVERSER_ID --input 'test' --memory 64"

# Test 9: Unload binaries
echo "??? Test 9: Unload Binaries ???"
TEMP_ID=$($CLIENT load --path ./plugins/reverser.wasm 2>/dev/null | grep "Binary ID:" | awk '{print $3}')
run_test "Unload binary" "$CLIENT unload --binary-id $TEMP_ID"

# Test 10: Persistence
echo "??? Test 10: Persistence ???"
echo "Restarting server to test persistence..."
kill $SERVER_PID
sleep 2
cargo run --release -p wasm-core &
SERVER_PID=$!
sleep 3
run_test "Binaries persist after restart" "$CLIENT list"

echo "??????????????????????????????????????????"
echo "?? STEP 4: Test Results"
echo "??????????????????????????????????????????"
echo ""

TOTAL_TESTS=$((TESTS_PASSED + TESTS_FAILED))

echo "Total Tests: $TOTAL_TESTS"
echo -e "${GREEN}Passed: $TESTS_PASSED${NC}"
echo -e "${RED}Failed: $TESTS_FAILED${NC}"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}??????????????????????????????????????????${NC}"
    echo -e "${GREEN}?? ALL TESTS PASSED! ??${NC}"
    echo -e "${GREEN}??????????????????????????????????????????${NC}"
    exit 0
else
    echo -e "${RED}??????????????????????????????????????????${NC}"
    echo -e "${RED}? SOME TESTS FAILED ?${NC}"
    echo -e "${RED}??????????????????????????????????????????${NC}"
    exit 1
fi
