#!/bin/bash

GREEN=$'\033[0;32m'
RED=$'\033[0;31m'
RESET=$'\033[0m'

VIT="$(cd "$(dirname "$0")/.." && pwd)/target/release/vit"

now_ms() {
    perl -MTime::HiRes=time -e 'printf "%d", time * 1000'
}

read_cached() {
    [ -f "$CACHE" ] && grep "^$1:" "$CACHE" 2>/dev/null | cut -d: -f2
}

format_delta() {
    local ms=$1 old=$2
    local pct=$(( (ms - old) * 100 / old ))
    if [ $pct -le 0 ]; then
        echo "${GREEN}${pct}%${RESET}"
    else
        echo "${RED}+${pct}%${RESET}"
    fi
}

bench() {
    local name="$1"; shift
    local start=$(now_ms)
    "$@" >/dev/null 2>&1
    local ms=$(( $(now_ms) - start ))
    local old=$(read_cached "$name")
    printf "  %-20s %6sms" "$name" "$ms"
    [ -n "$old" ] && echo -n "  $(format_delta $ms $old)"
    echo ""
    echo "$name:$ms" >> "${CACHE}.tmp"
}

bench_done() {
    mv "${CACHE}.tmp" "$CACHE"
    echo ""
}
