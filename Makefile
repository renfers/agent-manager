.PHONY: build test fmt lint clean run dry

build:
	cargo build --release

test:
	cargo test --all-features

fmt:
	cargo fmt

lint:
	cargo clippy -- -D warnings

clean:
	cargo clean

run:
	cargo run -- --workflow constellation-chat

dry:
	cargo run -- --workflow constellation-chat --dry-run
