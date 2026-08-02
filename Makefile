.PHONY: format format-check check lint test verify pre-commit-install

format:
	cargo fmt --all

format-check:
	cargo fmt --all -- --check

check:
	cargo check --workspace --all-targets

lint:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

test:
	cargo test --workspace --all-targets

verify: format-check check lint test

pre-commit-install:
	pre-commit install
