.PHONY: build dev

build:
	cargo build --release

dev:
	cargo clippy -- -D warnings
	cargo build --release

