# !/bin/sh

commit () {
	git commit -q --allow-empty -m "$1"
}

setup_basic_repo() {
    git init -q .
    git config user.name tester
    git config user.email tester@vit

    commit "fix parser bug"
    commit "parser handles nested expressions"
    commit "add lexer tokens"
    commit "improve lexer speed"
    commit "update readme docs"
    commit "docs typo fix"
    commit "auth login cookie"
    commit "auth token refresh"
}
