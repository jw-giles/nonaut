.DEFAULT_GOAL := check

.PHONY: check fmt fmt-check lint test build run clean

check: fmt-check lint test

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

lint:
	cargo clippy --all-targets --all-features -- -D warnings

test:
	cargo test --all-targets --all-features

build:
	cargo build --release

run:
	cargo run --release

clean:
	cargo clean
