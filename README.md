# Extism JavaScript PDK

![GitHub License](https://img.shields.io/github/license/extism/extism)
![GitHub release (with filter)](https://img.shields.io/github/v/release/extism/js-pdk)

Build [Extism Plug-ins](https://extism.org/docs/concepts/plug-in) in JavaScript. The `extism-js` compiler takes your JS source and compiles it to a Wasm module using [QuickJS-ng](https://github.com/quickjs-ng/quickjs) (via [rquickjs](https://github.com/DelSkayn/rquickjs)) and [Wizer](https://github.com/bytecodealliance/wizer).

## How This Runtime Differs from Node.js / Browsers

Extism JS plugins run inside a WebAssembly sandbox. There is no event loop, no I/O, and no access to the operating system. This means:

- **Synchronous execution model.** Your exported function runs to completion and returns. There is no task queue, no `setTimeout`, and no background work.
- **`async`/`await` works**, but only over values that are already resolved. `fetch()` returns a `Promise`, but the underlying HTTP call completes synchronously before it's handed to you. This means libraries that use `await fetch(...)` will work, but nothing actually runs concurrently.
- **No Node.js APIs.** No `fs`, `path`, `net`, `child_process`, etc. (`Buffer` is available as a polyfill.)
- **No browser-specific APIs.** No DOM, `window`, `localStorage`, `Worker`, `WebSocket`, etc.
- **ES2020 language features.** The QuickJS-ng engine supports up to ES2020 syntax (nullish coalescing, optional chaining, BigInt, `Promise.allSettled`, etc.). Target `es2020` in your bundler.
- **CommonJS modules.** Use `module.exports` when not using a bundler. With a bundler, you can write ESM and compile to CJS.

The PDK provides a curated set of Web-standard APIs (see table below) alongside Extism-specific APIs for host communication. Many npm packages that are pure JavaScript will work out of the box when bundled. Packages that depend on Node.js built-ins or browser APIs will not.

## Supported APIs

### Web Standard APIs

| API | Support | Notes |
|---|---|---|
| `fetch()` | Full | Wraps Extism HTTP; supports `Request` init, returns `Response` with `.text()`, `.json()`, `.arrayBuffer()` |
| `Headers` | Full | Case-insensitive, `append`/`delete`/`get`/`has`/`set`/`forEach`/`entries`/`keys`/`values` |
| `Response` | Partial | No `.blob()` or `.formData()`; no streaming |
| `URL` | Full | Spec-compliant via core-js polyfill |
| `URLSearchParams` | Full | Spec-compliant via core-js polyfill |
| `URLPattern` | Full | Via urlpattern-polyfill |
| `TextEncoder` | Full | `.encode()` and `.encodeInto()` (UTF-8 only) |
| `TextDecoder` | Partial | UTF-8 only; no streaming mode |
| `console` | Full | `.log` `.info` `.warn` `.error` `.debug` `.trace` `.assert` `.time`/`.timeEnd`/`.timeLog` `.count`/`.countReset` `.table` |
| `atob` / `btoa` | Full | Throws `DOMException` on invalid input |
| `structuredClone` | Partial | Primitives, Date, RegExp, ArrayBuffer, TypedArrays, Map, Set, Array, Error, plain objects. No DOM nodes, functions, or symbols. |
| `crypto.getRandomValues()` | Full | Max 65,536 bytes; integer TypedArrays only |
| `crypto.randomUUID()` | Full | RFC 4122 v4 |
| `crypto.subtle.digest()` | Partial | SHA-1, SHA-256, SHA-384, SHA-512 only. No encrypt/decrypt/sign/verify/key operations. |
| `performance.now()` | Full | Millisecond precision via WASI clock |
| `performance.timeOrigin` | Full | |
| `DOMException` | Full | Standard `name`/`message`/`code` properties |
| `Event` | Full | Constructor with `bubbles`/`cancelable` options, `preventDefault`, `stopPropagation`, `stopImmediatePropagation` |
| `EventTarget` | Full | `addEventListener` (with `once`), `removeEventListener`, `dispatchEvent` |
| `queueMicrotask` | Sync | Executes the callback immediately (no event loop) |
| `globalThis.self` | Full | Alias for `globalThis` |
| `Date` | Full | Host-provided current time via WASI |
| `JSON` / `Math` / `RegExp` / `Promise` / `Proxy` / `Reflect` | Full | ES2020 standard library |
| `Map` / `Set` / `WeakMap` / `WeakSet` | Full | |
| `ArrayBuffer` / `DataView` / Typed Arrays | Full | All standard TypedArray types |
| `BigInt` | Full | |
| `Buffer` | Full | Node.js-compatible. `from`/`alloc`/`concat`/`isBuffer`, all encodings (utf8, hex, base64, base64url, latin1, ascii), read/write integer methods, `slice`, `copy`, `indexOf`, `fill`, `equals`, `compare`. Works with npm packages that use `require('buffer')` via esbuild alias. |

### Extism PDK APIs

| API | Description |
|---|---|
| `Host.inputString()` / `Host.inputBytes()` | Read plug-in input |
| `Host.outputString(s)` / `Host.outputBytes(buf)` | Set plug-in output |
| `Host.getFunctions()` | Access host-provided functions |
| `Config.get(key)` | Read host-provided configuration |
| `Var.getString(key)` / `Var.getBytes(key)` / `Var.set(key, val)` | Persistent key-value storage across calls |
| `Http.request(req)` | Low-level synchronous HTTP (prefer `fetch()`) |
| `Memory.fromString(s)` / `Memory.fromBuffer(buf)` / `Memory.find(offset)` | Manual memory management for host function interop |
| `module.exports = { fn }` | Export functions callable by the host |

### Not Available

`setTimeout` / `setInterval`, `fs`, `path`, `net`, `child_process`, `Worker`, `WebSocket`, DOM APIs, `localStorage`, Streams, `Canvas`, `import()` dynamic imports.

## Install

### Linux, macOS

```bash
curl -O https://raw.githubusercontent.com/extism/js-pdk/main/install.sh
bash install.sh
```

### Windows

> [7zip](https://www.7-zip.org/) is required.

```bash
powershell Invoke-WebRequest -Uri https://raw.githubusercontent.com/extism/js-pdk/main/install-windows.ps1 -OutFile install-windows.ps1
powershell -executionpolicy bypass -File .\install-windows.ps1
```

### Dependencies

[Binaryen](https://github.com/WebAssembly/binaryen) (`wasm-merge` and `wasm-opt`) must be on your `PATH`. Install with `brew install binaryen` on macOS, or grab a release from the [Binaryen releases page](https://github.com/WebAssembly/binaryen/releases).

Verify the install:

```
extism-js --help
```

> **Note**: On macOS you may need to allow the unsigned binary in System Settings > Privacy & Security.

## Getting Started

### Exports

Write a `plugin.js` that exports functions the host can call:

```javascript
function greet() {
  const name = Host.inputString();
  Host.outputString(`Hello, ${name}!`);
}

module.exports = { greet };
```

Declare the Wasm interface in `plugin.d.ts`:

```typescript
declare module "main" {
  export function greet(): I32;
}
```

Compile and run:

```bash
extism-js plugin.js -i plugin.d.ts -o plugin.wasm
extism call plugin.wasm greet --input="Benjamin" --wasi
# => Hello, Benjamin!
```

> **Note**: `--wasi` is currently required for all JavaScript plug-ins.

### Error Handling

Thrown exceptions are returned as errors to the host:

```javascript
function greet() {
  const name = Host.inputString();
  if (name === "Benjamin") {
    throw new Error("Sorry, we don't greet Benjamins!");
  }
  Host.outputString(`Hello, ${name}!`);
}

module.exports = { greet };
```

```bash
extism call plugin.wasm greet --input="Benjamin" --wasi
# => Error: Uncaught Error: Sorry, we don't greet Benjamins!
echo $?
# => 1
```

### JSON

Use `JSON.parse` and `JSON.stringify` for complex types:

```javascript
function sum() {
  const params = JSON.parse(Host.inputString());
  Host.outputString(JSON.stringify({ sum: params.a + params.b }));
}

module.exports = { sum };
```

```bash
extism call plugin.wasm sum --input='{"a": 20, "b": 21}' --wasi
# => {"sum":41}
```

### Using fetch

The `fetch()` API wraps the Extism HTTP interface and works with both `.then()` and `async`/`await`:

```javascript
async function callApi() {
  const response = await fetch("https://jsonplaceholder.typicode.com/todos/1");
  if (!response.ok) {
    throw new Error(`HTTP ${response.status}`);
  }
  const todo = await response.json();
  Host.outputString(`Todo: ${todo.title}`);
}

module.exports = { callApi };
```

> The host must allow the target domain via `--allow-host`.

### Configs

Read host-provided key-value configuration with `Config.get`:

```javascript
function greet() {
  const user = Config.get("user");
  Host.outputString(`Hello, ${user}!`);
}

module.exports = { greet };
```

```bash
extism call plugin.wasm greet --config user=Benjamin --wasi
# => Hello, Benjamin!
```

### Variables

Mutable key-value storage that persists across function calls within a plug-in's lifetime:

```javascript
function count() {
  let count = parseInt(Var.getString("count") || "0", 10);
  count += 1;
  Var.set("count", count.toString());
  Host.outputString(count.toString());
}

module.exports = { count };
```

### Logging

```javascript
function logStuff() {
  console.log("Info-level log");
  console.debug("Debug-level log");
  console.warn("Warning");
  console.error("Error");
  console.table([{ name: "Alice", age: 30 }, { name: "Bob", age: 25 }]);
}

module.exports = { logStuff };
```

```bash
extism call plugin.wasm logStuff --wasi --log-level=debug
```

### Host Functions

Declare host functions in your `.d.ts` file and call them through `Host.getFunctions()`:

```typescript
declare module "main" {
  export function greet(): I32;
}

declare module "extism:host" {
  interface user {
    myHostFunction1(ptr: I64): I64;
    myHostFunction2(ptr: I64): I64;
  }
}
```

```javascript
const { myHostFunction1, myHostFunction2 } = Host.getFunctions();

function greet() {
  let mem = Memory.fromString("Hello from JS");
  let offset = myHostFunction1(mem.offset);
  let response = Memory.find(offset).readString();
  Host.outputString(response);
}

module.exports = { greet };
```

> Host functions accept up to 5 `I64` arguments. You manage memory manually using the `Memory` API.

## Using with a Bundler

Use a bundler to write in TypeScript, use ESM syntax, or import npm packages. Two constraints:

1. Output must be **CJS format**
2. Target must be **es2020** or lower

### esbuild Setup

```bash
mkdir extism-plugin && cd extism-plugin
npm init -y
npm install esbuild @extism/js-pdk --save-dev
mkdir src dist
```

Add a `jsconfig.json` or `tsconfig.json` for IDE support:

```jsonc
{
  "compilerOptions": {
    "lib": [],
    "types": ["@extism/js-pdk"],
    "noEmit": true
  },
  "include": ["src/**/*"]
}
```

Add `esbuild.js`:

```js
const esbuild = require("esbuild");

esbuild.build({
  entryPoints: ["src/index.js"],
  outdir: "dist",
  bundle: true,
  sourcemap: true,
  minify: false,
  format: "cjs",
  target: ["es2020"],
});
```

Add a build script to `package.json`:

```json
{
  "scripts": {
    "build": "node esbuild.js && extism-js dist/index.js -i src/index.d.ts -o dist/plugin.wasm"
  }
}
```

Now you can use ESM imports and npm packages:

```js
import { closest } from "fastest-levenshtein";

export function get_closest() {
  let input = Host.inputString();
  Host.outputString(closest(input, ["slow", "faster", "fastest"]));
}
```

```bash
npm install fastest-levenshtein
npm run build
extism call dist/plugin.wasm get_closest --input="fest" --wasi
# => fastest
```

### React / JSX / TSX

You can use React for server-side rendering in plug-ins:

```bash
npm install react-dom --save
npm install @types/react --save-dev
```

```tsx
import { renderToString } from "react-dom/server";
import React from "react";

function App({ name }: { name: string }) {
  return <p>Hello {name}!</p>;
}

export function render() {
  const props = JSON.parse(Host.inputString());
  Host.outputString(renderToString(<App {...props} />));
}
```

See [examples/react](./examples/react/) for a complete example.

## Generating Bindings

[XTP Bindgen](https://github.com/dylibso/xtp-bindgen) can generate type-safe PDK bindings from an OpenAPI-inspired schema:

```yaml
version: v1-draft
exports:
  CountVowels:
    input:
      type: string
      contentType: text/plain; charset=utf-8
    output:
      $ref: "#/components/schemas/VowelReport"
      contentType: application/json
```

```bash
xtp plugin init --schema-file ./example-schema.yaml
# Select "TypeScript" and implement the generated stubs
xtp plugin build
```

See the [XTP Schema docs](https://docs.xtp.dylibso.com/docs/concepts/xtp-schema) for more.

## How It Works

JavaScript can't compile directly to Wasm because it doesn't have the right type system. Instead, the `extism-js` compiler:

1. Loads an engine Wasm module containing the QuickJS-ng runtime
2. Initializes a QuickJS context and loads your JS source code
3. Parses your exports and generates 1-to-1 Wasm proxy functions
4. Snapshots the initialized state with Wizer and emits a new Wasm file

The result is a self-contained Wasm module that can be used with any Extism host SDK.

## Compiling from Source

### Prerequisites

1. [Rust](https://rustup.rs) with `rustup target add --toolchain stable wasm32-wasip1`
2. WASI SDK: `make download-wasi-sdk`
3. [CMake](https://cmake.org/install/) (`brew install cmake` on macOS)
4. [Binaryen](https://github.com/WebAssembly/binaryen/) on your PATH
5. [7zip](https://www.7-zip.org/) (Windows only)

### Build

```bash
make        # builds core engine + CLI
make test   # compiles examples and runs test suite
```
