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
		cd ./examples/host_funcs && go run . ../host_funcs.wasm
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
		@extism call examples/base64.wasm greet --wasi --input="Benjamin" --log-level=debug
		@error_msg=$$(extism call examples/try-catch.wasm greet --wasi --input="Benjamin" --log-level debug 2>&1); \
		if echo "$$error_msg" | grep -q "got error"; then \
			echo "Test passed - found expected error"; \
		else \
			echo "Test failed - did not find expected error message"; \
			echo "Got: $$error_msg"; \
			exit 1; \
		fi
		@output=$$(extism call examples/atob_btoa.wasm greet --wasi --log-level debug 2>&1); \
		if echo "$$output" | grep -q "all tests passed"; then \
			echo "Test passed - atob/btoa"; \
		else \
			echo "Test failed - atob/btoa"; \
			echo "Got: $$output"; \
			exit 1; \
		fi
		@output=$$(extism call examples/structured_clone.wasm greet --wasi --log-level debug 2>&1); \
		if echo "$$output" | grep -q "all tests passed"; then \
			echo "Test passed - structuredClone"; \
		else \
			echo "Test failed - structuredClone"; \
			echo "Got: $$output"; \
			exit 1; \
		fi
		@output=$$(extism call examples/perf.wasm greet --wasi --log-level debug 2>&1); \
		if echo "$$output" | grep -q "all tests passed"; then \
			echo "Test passed - performance"; \
		else \
			echo "Test failed - performance"; \
			echo "Got: $$output"; \
			exit 1; \
		fi
		@output=$$(extism call examples/console_extra.wasm greet --wasi --log-level debug 2>&1); \
		if echo "$$output" | grep -q "all tests passed"; then \
			echo "Test passed - console extras"; \
		else \
			echo "Test failed - console extras"; \
			echo "Got: $$output"; \
			exit 1; \
		fi
		@output=$$(extism call examples/encode_into.wasm greet --wasi --log-level debug 2>&1); \
		if echo "$$output" | grep -q "all tests passed"; then \
			echo "Test passed - TextEncoder.encodeInto"; \
		else \
			echo "Test failed - TextEncoder.encodeInto"; \
			echo "Got: $$output"; \
			exit 1; \
		fi
		@output=$$(extism call examples/fetch.wasm greet --wasi --allow-host "example.com" --log-level debug 2>&1); \
		if echo "$$output" | grep -q "all tests passed"; then \
			echo "Test passed - fetch"; \
		else \
			echo "Test failed - fetch"; \
			echo "Got: $$output"; \
			exit 1; \
		fi
		@output=$$(extism call examples/crypto.wasm greet --wasi --log-level debug 2>&1); \
		if echo "$$output" | grep -q "all tests passed"; then \
			echo "Test passed - crypto"; \
		else \
			echo "Test failed - crypto"; \
			echo "Got: $$output"; \
			exit 1; \
		fi
		@output=$$(extism call examples/compat.wasm greet --wasi --log-level debug 2>&1); \
		if echo "$$output" | grep -q "all tests passed"; then \
			echo "Test passed - compat"; \
		else \
			echo "Test failed - compat"; \
			echo "Got: $$output"; \
			exit 1; \
		fi
		@output=$$(extism call examples/event.wasm greet --wasi --log-level debug 2>&1); \
		if echo "$$output" | grep -q "all tests passed"; then \
			echo "Test passed - event"; \
		else \
			echo "Test failed - event"; \
			echo "Got: $$output"; \
			exit 1; \
		fi
		@output=$$(extism call examples/console_table.wasm greet --wasi --log-level debug 2>&1); \
		if echo "$$output" | grep -q "all tests passed"; then \
			echo "Test passed - console_table"; \
		else \
			echo "Test failed - console_table"; \
			echo "Got: $$output"; \
			exit 1; \
		fi
		@output=$$(extism call examples/subtle_digest.wasm greet --wasi --log-level debug 2>&1); \
		if echo "$$output" | grep -q "all tests passed"; then \
			echo "Test passed - subtle_digest"; \
		else \
			echo "Test failed - subtle_digest"; \
			echo "Got: $$output"; \
			exit 1; \
		fi
		@output=$$(extism call examples/buffer.wasm greet --wasi --log-level debug 2>&1); \
		if echo "$$output" | grep -q "all tests passed"; then \
			echo "Test passed - buffer"; \
		else \
			echo "Test failed - buffer"; \
			echo "Got: $$output"; \
			exit 1; \
		fi
		@output=$$(extism call examples/buffer_npm.wasm greet --wasi --log-level debug 2>&1); \
		if echo "$$output" | grep -q "all tests passed"; then \
			echo "Test passed - buffer_npm"; \
		else \
			echo "Test failed - buffer_npm"; \
			echo "Got: $$output"; \
			exit 1; \
		fi
		@output=$$(extism call examples/async_exception.wasm greet --wasi 2>&1); \
		if echo "$$output" | grep -q "wasm error: unreachable"; then \
			echo "Test failed - async_exception: got panic instead of clean error"; \
			echo "Output: $$output"; \
			exit 1; \
		elif echo "$$output" | grep -q "not a function"; then \
			echo "Test passed - async_exception: got clean error"; \
		else \
			echo "Test failed - async_exception: unexpected output"; \
			echo "Output: $$output"; \
			exit 1; \
		fi

compile-examples: cli
		cd examples/react && npm install && npm run build && cd ../..
		./target/release/extism-js examples/simple_js/script.js -i examples/simple_js/script.d.ts -o examples/simple_js.wasm
		cd examples/bundled && npm install && npm run build && cd ../..
		./target/release/extism-js examples/host_funcs/script.js -i examples/host_funcs/script.d.ts -o examples/host_funcs.wasm
		./target/release/extism-js examples/exports/script.js -i examples/exports/script.d.ts -o examples/exports.wasm
		./target/release/extism-js examples/exception/script.js -i examples/exception/script.d.ts -o examples/exception.wasm
		./target/release/extism-js examples/console/script.js -i examples/console/script.d.ts -o examples/console.wasm
		./target/release/extism-js examples/base64/script.js -i examples/base64/script.d.ts -o examples/base64.wasm
		./target/release/extism-js examples/try-catch/script.js -i examples/try-catch/script.d.ts -o examples/try-catch.wasm
		./target/release/extism-js examples/atob_btoa/script.js -i examples/atob_btoa/script.d.ts -o examples/atob_btoa.wasm
		./target/release/extism-js examples/structured_clone/script.js -i examples/structured_clone/script.d.ts -o examples/structured_clone.wasm
		./target/release/extism-js examples/perf/script.js -i examples/perf/script.d.ts -o examples/perf.wasm
		./target/release/extism-js examples/console_extra/script.js -i examples/console_extra/script.d.ts -o examples/console_extra.wasm
		./target/release/extism-js examples/encode_into/script.js -i examples/encode_into/script.d.ts -o examples/encode_into.wasm
		./target/release/extism-js examples/fetch/script.js -i examples/fetch/script.d.ts -o examples/fetch.wasm
		./target/release/extism-js examples/crypto/script.js -i examples/crypto/script.d.ts -o examples/crypto.wasm
		./target/release/extism-js examples/compat/script.js -i examples/compat/script.d.ts -o examples/compat.wasm
		./target/release/extism-js examples/event/script.js -i examples/event/script.d.ts -o examples/event.wasm
		./target/release/extism-js examples/console_table/script.js -i examples/console_table/script.d.ts -o examples/console_table.wasm
		./target/release/extism-js examples/subtle_digest/script.js -i examples/subtle_digest/script.d.ts -o examples/subtle_digest.wasm
		./target/release/extism-js examples/buffer/script.js -i examples/buffer/script.d.ts -o examples/buffer.wasm
		cd examples/buffer_npm && npm install && node esbuild.js && cd ../..
		./target/release/extism-js examples/buffer_npm/dist/index.js -i examples/buffer_npm/src/index.d.ts -o examples/buffer_npm.wasm
		./target/release/extism-js examples/async_exception/script.js -i examples/async_exception/script.d.ts -o examples/async_exception.wasm

kitchen: 
	cd examples/kitchen-sink && npm install && npm run build && cd ../..
	./target/release/extism-js examples/kitchen-sink/dist/index.js -i examples/kitchen-sink/src/index.d.ts -o examples/kitchen-sink.wasm
	@extism call examples/kitchen-sink.wasm greet --input "Steve" --wasi --allow-host "*" --config "last_name=Manuel"
