.DEFAULT_GOAL := check

.PHONY: check fmt fmt-check lint test integration build smoke run clean

check: fmt-check lint test

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

lint:
	cargo clippy --all-targets --all-features -- -D warnings

test:
	cargo test --all-targets --all-features

integration: build
	cargo test --test harness -- --ignored --test-threads=1

build:
	cargo build --release

smoke: build
	./target/release/nonaut exec -- "printf nonaut-smoke" | grep -q 'nonaut-smoke'

run:
	cargo run --release

clean:
	cargo clean
