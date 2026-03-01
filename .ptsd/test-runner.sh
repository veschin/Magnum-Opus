#!/bin/bash
# PTSD test runner wrapper for Rust cargo test
# Translates cargo test output to PTSD-compatible format
set -e

FEATURE="${1:-}"
CRATE_DIR="magnum_opus"

if [ -z "$FEATURE" ]; then
    FILTER=""
else
    # Convert feature name to test module filter
    FILTER=$(echo "$FEATURE" | tr '-' '_')
    FILTER="${FILTER}_bdd"
fi

cd "$CRATE_DIR"
OUTPUT=$(cargo test "$FILTER" 2>&1 || true)

# Parse "test result:" line
RESULT_LINE=$(echo "$OUTPUT" | grep "^test result:" || echo "")
if [ -z "$RESULT_LINE" ]; then
    echo "pass:0 fail:0"
    exit 1
fi

PASSED=$(echo "$RESULT_LINE" | grep -oP '\d+ passed' | grep -oP '\d+')
FAILED=$(echo "$RESULT_LINE" | grep -oP '\d+ failed' | grep -oP '\d+')

echo "pass:${PASSED:-0} fail:${FAILED:-0}"

# Print individual test results for PTSD
echo "$OUTPUT" | grep "^test " | while read -r line; do
    TEST_NAME=$(echo "$line" | awk '{print $2}')
    STATUS=$(echo "$line" | awk '{print $NF}')
    if [ "$STATUS" = "ok" ]; then
        echo "PASS: $TEST_NAME"
    elif [ "$STATUS" = "FAILED" ]; then
        echo "FAIL: $TEST_NAME"
    fi
done
