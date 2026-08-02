.PHONY: format format-check check lint test verify pre-commit-install pre-commit

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
	uv run pre-commit install

pre-commit:
	uv run pre-commit run --all-files
