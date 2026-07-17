TDIR="$(cd "$(dirname "$0")" && pwd)"
. "$TDIR"/lib-test.sh
. "$TDIR"/repo-lib.sh

setup_test_dir "$(basename "$0")"

setup_basic_repo

test_expect_success 'near without index asks to run map' '
	vit near parser 2>&1 | grep -q "no index found"
'

test_expect_success 'near without index exits nonzero' '
	! vit near parser >/dev/null 2>&1
'

test_expect_success 'setup map' '
    vit map >/dev/null 2>&1
'

test_expect_success 'near finds commits about the query' '
	vit near parser | grep -q "parser"
'

test_expect_success 'near respects the result limit' '
	test "$(vit near parser -1 | wc -l)" -eq 1
'

test_expect_success 'near rejects a split message' '
	! vit near foo -v bar >/dev/null 2>&1
'

test_done
