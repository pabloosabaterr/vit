#!/bin/bash
. "$(dirname "$0")/lib-bench.sh"

REPO="./"
CACHE="$(cd "$(dirname "$0")" && pwd)/.bench_cache_vit"

COMMITS=$(git rev-list --all --count 2>/dev/null || echo "?")
echo " ("Here bro", $COMMITS commits)"
echo ""

bench "map"   "$VIT" map
bench "near"  "$VIT" near "vit vector" -5

bench_done
