#!/bin/sh

TDIR="$(cd "$(dirname "$0")" && pwd)"
. "$TDIR"/lib-test.sh
. "$TDIR"/repo-lib.sh

setup_test_dir "$(basename "$0")"

setup_basic_repo

test_expect_success 'version prints vit and four numbers' '
	vit -V | grep -Eq "^vit [0-9]+\.[0-9]+\.[0-9]+\.[0-9]+$"
'

test_expect_success 'unknown command dies' '
	! vit test 2>msg &&
	grep -q "fatal: unknown command" msg
'

test_expect_success 'config writes dims to .vitrc' '
	vit config dims 16 &&
	grep -q "dims=16" .vitrc
'


cd "$TDIR" && rm -rf "$TRASH"

test_done
