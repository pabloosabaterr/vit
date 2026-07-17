.PHONY: build dev
.SILENT: test

build:
	cargo build --release

dev:
	cargo clippy -- -D warnings
	cargo build --release

test:
	cd t && \
    sh runner.sh
