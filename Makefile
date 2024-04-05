.PHONY: cli core fmt clean
.DEFAULT_GOAL := cli

download-wasi-sdk:
	sh install-wasi-sdk.sh

install:
	cargo install --path crates/cli

cli: core
		cd crates/cli && cargo build --release && cd -

core:
		cd crates/core \
			  && cd src/prelude \
				&& npm install \
				&& npm run build \
				&& npx -y -p typescript tsc src/index.ts --lib es2020 --declaration --emitDeclarationOnly --outDir dist \
				&& cd ../.. \
				&& cargo build --release --target=wasm32-wasi \
				&& cd -

fmt: fmt-core fmt-cli

fmt-core:
		cd crates/core/ \
				&& cargo fmt -- --check \
				&& cargo clippy --target=wasm32-wasi -- -D warnings \
				&& cd -

fmt-cli:
		cd crates/cli/ \
				&& cargo fmt -- --check \
				&& cargo clippy -- -D warnings \
				&& cd -

clean: clean-wasi-sdk clean-cargo

clean-cargo:
		cargo clean

clean-wasi-sdk:
		rm -r wasi-sdk 2> /dev/null || true

test: compile-examples
		@extism call examples/simple_js.wasm greet --wasi --input="Benjamin"
		@extism call examples/bundled.wasm greet --wasi --input="Benjamin" --allow-host "example.com"
		@python3 -m venv ./.venv && \
			. ./.venv/bin/activate && \
			pip install -r examples/host_funcs/requirements.txt && \
			python3 examples/host_funcs/host.py examples/host_funcs.wasm && \
			deactivate

compile-examples: cli
		./target/release/extism-js examples/simple_js/script.js -i examples/simple_js/script.d.ts -o examples/simple_js.wasm
		cd examples/bundled && npm install && npm run build && cd ../..
		./target/release/extism-js examples/host_funcs/script.js -i examples/host_funcs/script.d.ts -o examples/host_funcs.wasm
		./target/release/extism-js examples/exports/script.js -i examples/exports/script.d.ts -o examples/exports.wasm
