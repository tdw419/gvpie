#!/usr/bin/env bash
set -euo pipefail

echo "🔍 Validating GVPIE I/O contract..."

WORKDIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RUST_FILE="$WORKDIR/src/io_contract.rs"
WGSL_FILE="$WORKDIR/shaders/contract.wgsl"

if [[ ! -f "$RUST_FILE" || ! -f "$WGSL_FILE" ]]; then
    echo "❌ Contract files missing. Expected:"
    echo "   - $RUST_FILE"
    echo "   - $WGSL_FILE"
    exit 1
fi

echo "📋 Comparing Rust and WGSL constant names..."
rust_consts=$(grep -E 'pub const [A-Z0-9_]+' "$RUST_FILE" | sed -E 's/.*pub const ([A-Z0-9_]+).*/\1/' | sort)
wgsl_consts=$(grep -E 'const [A-Z0-9_]+' "$WGSL_FILE" | sed -E 's/.*const ([A-Z0-9_]+).*/\1/' | sort)

diff_output=$(diff <(echo "$rust_consts") <(echo "$wgsl_consts") || true)
if [[ -n "$diff_output" ]]; then
    echo "❌ Contract constant mismatch detected:"
    echo "$diff_output"
    exit 1
fi
echo "✅ Constant names match"

echo "📏 Running contract validation tests..."
cargo test -- --quiet

echo "✅ Contract validation complete"
