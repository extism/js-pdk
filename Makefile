.PHONY: cli core fmt clean
.DEFAULT_GOAL := cli

download-wasi-sdk:
ifeq ($(OS),Windows_NT)
	powershell -executionpolicy bypass -File .\install-wasi-sdk.ps1
else
	sh install-wasi-sdk.sh
endif

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
				&& cargo build --release --target=wasm32-wasip1 \
				&& wasm-opt --enable-reference-types --enable-bulk-memory --strip -O3 ../../target/wasm32-wasip1/release/js_pdk_core.wasm -o ../../target/wasm32-wasip1/release/js_pdk_core.wasm \
				&& cd -

fmt: fmt-core fmt-cli

fmt-core:
		cd crates/core/ \
				&& cargo fmt -- --check \
				&& cargo clippy --target=wasm32-wasip1 -- -D warnings \
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
ifeq ($(OS),Windows_NT)
		@python3 -m venv ./.venv && \
			./.venv/Scripts/activate.bat && \
			pip install -r examples/host_funcs/requirements.txt && \
			python examples/host_funcs/host.py examples/host_funcs.wasm && \
			./.venv/Scripts/deactivate.bat
else
		@python3 -m venv ./.venv && \
			. ./.venv/bin/activate && \
			pip install -r examples/host_funcs/requirements.txt && \
			python3 examples/host_funcs/host.py examples/host_funcs.wasm && \
			deactivate
endif
		@extism call examples/react.wasm render --wasi
		@extism call examples/react.wasm setState --input='{"action": "SET_SETTING", "payload": { "backgroundColor": "tomato" }}' --wasi
		@error_msg=$$(extism call examples/exception.wasm greet --wasi --input="Benjamin" 2>&1); \
		if echo "$$error_msg" | grep -q "shibboleth"; then \
			echo "Test passed - found expected error"; \
		else \
			echo "Test failed - did not find expected error message"; \
			echo "Got: $$error_msg"; \
			exit 1; \
		fi
		@extism call examples/console.wasm greet --wasi --input="Benjamin" --log-level=debug

compile-examples: cli
		cd examples/react && npm install && npm run build && cd ../..
		./target/release/extism-js examples/simple_js/script.js -i examples/simple_js/script.d.ts -o examples/simple_js.wasm
		cd examples/bundled && npm install && npm run build && cd ../..
		./target/release/extism-js examples/host_funcs/script.js -i examples/host_funcs/script.d.ts -o examples/host_funcs.wasm
		./target/release/extism-js examples/exports/script.js -i examples/exports/script.d.ts -o examples/exports.wasm
		./target/release/extism-js examples/exception/script.js -i examples/exception/script.d.ts -o examples/exception.wasm
		./target/release/extism-js examples/console/script.js -i examples/console/script.d.ts -o examples/console.wasm

kitchen: 
	cd examples/kitchen-sink && npm install && npm run build && cd ../..
	./target/release/extism-js examples/kitchen-sink/dist/index.js -i examples/kitchen-sink/src/index.d.ts -o examples/kitchen-sink.wasm
	@extism call examples/kitchen-sink.wasm greet --input "Steve" --wasi --allow-host "*" --config "last_name=Manuel"
