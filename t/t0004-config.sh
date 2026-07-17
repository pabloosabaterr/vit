TDIR="$(cd "$(dirname "$0")" && pwd)"
. "$TDIR"/lib-test.sh
. "$TDIR"/repo-lib.sh

setup_test_dir "$(basename "$0")"

setup_basic_repo

test_expect_success 'config writes dims to .vitrc' '
	vit config dims 16 &&
	grep -q "dims=16" .vitrc
'

test_expect_failure 'config preserves comments in .vitrc' '
	cat >.vitrc <<EOF
# test comment that should not be removed
dims=10
EOF

	vit config dims 39

	grep -q "should not be removed" .vitrc
'

test_done
