.DEFAULT_GOAL := check

.PHONY: check fmt fmt-check lint test build smoke run clean

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

smoke: build
	printf '%s\n' '{"path":"Cargo.toml","start_line":1,"end_line":3}' | ./target/release/nonaut __nonaut_tool read_file | grep -q 'name = "nonaut"'

run:
	cargo run --release

clean:
	cargo clean
