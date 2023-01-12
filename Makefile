.PHONY: build

build:
	cd crates/core && cargo build --release --target wasm32-wasi && cd ../..
	cd crates/cli && cargo build --release && cd ../..
