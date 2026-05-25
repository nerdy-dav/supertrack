.PHONY: build run release clean

build:
	cargo build

run:
	cargo run

release:
	cargo build --release
	@echo "Binary at: target/release/supertrack"

clean:
	cargo clean
