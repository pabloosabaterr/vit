#!/bin/bash
. "$(dirname "$0")/lib-bench.sh"

REPO="/tmp/vit-bench-git"
CACHE="$(cd "$(dirname "$0")" && pwd)/.bench_cache_git"

if [ ! -d "$REPO" ]; then
    echo "  cloning git repo (one time only)..."
    git clone --bare https://github.com/git/git.git "$REPO" 2>/dev/null
fi

cd "$REPO"

COMMITS=$(git rev-list --all --count 2>/dev/null || echo "?")
echo " ($(basename $REPO), $COMMITS commits)"
echo ""

bench "map"   "$VIT" map
bench "near"  "$VIT" near "hash algorithm" -5

bench_done
