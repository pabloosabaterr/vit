#!/bin/sh

TDIR="$(cd "$(dirname "$0")" && pwd)"
. "$TDIR"/lib-test.sh

setup_test_dir "$(basename "$0")"

test_expect_success 'help shows map usage' '
	vit help map 2>&1 | grep -q "usage: vit map"
'

test_done

