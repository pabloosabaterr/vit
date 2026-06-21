#!/bin/bash

DIR="$(dirname "$0")"

cargo build --release

echo ""

for f in "$DIR"/b*.sh; do
    echo -n "- $(basename $f)"
    bash "$f"
done
