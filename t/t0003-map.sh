TDIR="$(cd "$(dirname "$0")" && pwd)"
. "$TDIR"/lib-test.sh
. "$TDIR"/repo-lib.sh

setup_test_dir "$(basename "$0")"

setup_basic_repo

test_expect_success 'map reports every commit mapped' '
	vit map 2>&1 | grep -q "mapped 8 commits"
'

test_expect_success 'map creates the index files' '
	test -f .vit/wordmap &&
	test -f .vit/commits &&
	test -f .vit/stats
'

test_expect_success 'map -l lists the commits' '
	vit map -l 2>&1 | grep -q "auth login cookie"
'

test_expect_success 'map rejects unknown options' '
	! vit map --frobnicate >/dev/null 2>&1
'

test_expect_success 'map warns when the corpus is too small' '
	mkdir tiny && (
		cd tiny &&
		git init -q . &&
		git config user.name t &&
		git config user.email t@t &&
		git commit -q --allow-empty -m "lonely commit" &&
		vit map 2>&1 | grep -q "not enough data"
	)
'

test_expect_success 'map outside a git repo finds no commits' '
	mkdir norepo && (
        export GIT_CEILING_DIRECTORIES="$(pwd)" &&
		cd norepo &&
		vit map 2>&1 | grep -q "no commits found"
	)
'

test_done
