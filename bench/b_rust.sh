#!/bin/bash
. "$(dirname "$0")/lib-bench.sh"

REPO="/tmp/vit-bench-rust"
CACHE="$(cd "$(dirname "$0")" && pwd)/.bench_cache_rust"

if [ ! -d "$REPO" ]; then
    echo "  cloning rust repo (one time only)..."
    git clone --bare https://github.com/rust-lang/rust.git "$REPO" 2>/dev/null
fi

cd "$REPO"

COMMITS=$(git rev-list --all --count 2>/dev/null || echo "?")
echo " ($(basename $REPO), $COMMITS commits)"
echo ""

bench "map"   "$VIT" map
bench "near"  "$VIT" near "parser lexer" -5

bench_done
