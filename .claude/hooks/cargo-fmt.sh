#!/bin/sh
# Auto-format Rust files after Edit/Write. Silent on success, non-blocking on failure.
INPUT=$(cat)
FILE_PATH=$(echo "$INPUT" | jq -r '.tool_input.file_path // empty')
case "$FILE_PATH" in
  *.rs) rustfmt --edition 2024 "$FILE_PATH" 2>/dev/null || true ;;
esac
exit 0
