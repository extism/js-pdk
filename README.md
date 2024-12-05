# Extism JavaScript PDK

![GitHub License](https://img.shields.io/github/license/extism/extism)
![GitHub release (with filter)](https://img.shields.io/github/v/release/extism/js-pdk)

This project contains a tool that can be used to create
[Extism Plug-ins](https://extism.org/docs/concepts/plug-in) in JavaScript.

## Overview

This PDK uses [QuickJS](https://bellard.org/quickjs/) and
[wizer](https://github.com/bytecodealliance/wizer) to run javascript as an
Extism Plug-in.

This is essentially a fork of [Javy](https://github.com/bytecodealliance/javy)
by Shopify. We may wish to collaborate and upstream some things to them. For the
time being I built this up from scratch using some of their crates, namely
quickjs-wasm-rs.

> Warning: This is a very bare-bones runtime. It's only for running pure JS code
> and it does not expose node APIs or the browser APIs. We have limited support
> for some W3C APIs (e.g. we support `Text{Encoder,Decoder}` but not `fetch`),
> but many modules you take from npm will not work out of the box. There is no
> support for node APIs or anything that makes syscalls typically. You'll need
> to polyfill any APIs with a pure JS implementation, which is often possible,
> but some things, such as controlling sockets, are not possible. Feel free to
> [file an issue](https://github.com/extism/js-pdk/issues/new) if you think an
> API should be supported though.

## Install the compiler

We release the compiler as native binaries you can download and run. Check the
[releases](https://github.com/extism/js-pdk/releases) page for the latest.

## Install Script

### Linux, macOS

```bash
curl -O https://raw.githubusercontent.com/extism/js-pdk/main/install.sh
bash install.sh
```

### Windows

> 7zip is required, you can find it [here](https://www.7-zip.org/).

Open the Command Prompt as Administrator, then run :

```bash
powershell Invoke-WebRequest -Uri https://raw.githubusercontent.com/extism/js-pdk/main/install-windows.ps1 -OutFile install-windows.ps1
powershell -executionpolicy bypass -File .\install-windows.ps1
```

This will install extism-js and binaryen dependency under `Program File` folder
(i.e. C:\Program Files\Binaryen and C:\Program Files\Extism). You must add these
paths to your PATH environment variable.

### Testing the Install

> _Note_: [Binaryen](https://github.com/WebAssembly/binaryen), specifically the
> `wasm-merge` and `wasm-opt` tools are required as a dependency. We will try to
> package this up eventually but for now it must be reachable on your machine.
> You can install on mac with `brew install binaryen` or see their
> [releases page](https://github.com/WebAssembly/binaryen/releases).

Then run command with `-h` to see the help:

```
extism-js 1.1.1
Extism JavaScript PDK Plugin Compiler

USAGE:
    extism-js [FLAGS] [OPTIONS] <input-js>

FLAGS:
    -h, --help        Prints help information
        --skip-opt    Skip final optimization pass
    -V, --version     Prints version information

OPTIONS:
    -i <interface-file>         [default: index.d.ts]
    -o <output>                 [default: index.wasm]

ARGS:
    <input-js>
```

> **Note**: If you are using mac, you may need to tell your security system this
> unsigned binary is fine. If you think this is dangerous, or can't get it to
> work, see the "compile from source" section below.

## Getting Started

The goal of writing an
[Extism plug-in](https://extism.org/docs/concepts/plug-in) is to compile your
JavaScript code to a Wasm module with exported functions that the host
application can invoke. The first thing you should understand is creating an
export.

### Exports

Let's write a simple program that exports a `greet` function which will take a
name as a string and return a greeting string. Paste this into a file
`plugin.js`:

```javascript
function greet() {
  const name = Host.inputString();
  Host.outputString(`Hello, ${name}`);
}

module.exports = { greet };
```

Some things to note about this code:

1. We can export functions by name using the normal `module.exports` object.
   This allows the host to invoke this function. Like a normal js module,
   functions cannot be seen from the outside without exporting them.
2. Currently, you must use
   [CJS Module syntax](https://nodejs.org/api/modules.html#modules-commonjs-modules)
   when not using a bundler. So the `export` keyword is not directly supported.
   See the [Using with a Bundler](#using-with-a-bundler) section for more.
3. In this PDK we code directly to the ABI. We get input from the using using
   `Host.input*` functions and we return data back with the `Host.output*`
   functions.

We must also describe the Wasm interface for our plug-in. We do this with a
typescript module DTS file. Here is our `plugin.d.ts` file:

```typescript
declare module "main" {
  // Extism exports take no params and return an I32
  export function greet(): I32;
}
```

Let's compile this to Wasm now using the `extism-js` tool:

```bash
extism-js plugin.js -i plugin.d.ts -o plugin.wasm
```

We can now test `plugin.wasm` using the
[Extism CLI](https://github.com/extism/cli)'s `run` command:

```bash
extism call plugin.wasm greet --input="Benjamin" --wasi
# => Hello, Benjamin!
```

> **Note**: Currently `wasi` must be provided for all JavaScript plug-ins even
> if they don't need system access, however we're looking at how to make this
> optional.

> **Note**: We also have a web-based, plug-in tester called the
> [Extism Playground](https://playground.extism.org/)

### More Exports: Error Handling

We catch any exceptions thrown and return them as errors to the host. Suppose we
want to re-write our greeting module to never greet Benjamins:

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

Now compile and run:

```bash
extism-js plugin.js -i plugin.d.ts -o plugin.wasm
extism call plugin.wasm greet --input="Benjamin" --wasi
# => Error: Uncaught Error: Sorry, we don't greet Benjamins!
# =>    at greet (script.js:4)
# =>    at <eval> (script.js)
echo $? # print last status code
# => 1
extism call plugin.wasm greet --input="Zach" --wasi
# => Hello, Zach!
echo $?
# => 0
```

### JSON

If you want to handle more complex types, the plug-in can input and output bytes
with `Host.inputBytes` and `Host.outputBytes` respectively. Those bytes can
represent any complex type. A common format to use is JSON:

```javascript
function sum() {
  const params = JSON.parse(Host.inputString());
  Host.outputString(JSON.stringify({ sum: params.a + params.b }));
}
```

```bash
extism call plugin.wasm sum --input='{"a": 20, "b": 21}' --wasi
# => {"sum":41}
```

### Configs

Configs are key-value pairs that can be passed in by the host when creating a
plug-in. These can be useful to statically configure the plug-in with some data
that exists across every function call. Here is a trivial example using
`Config.get`:

```javascript
function greet() {
  const user = Config.get("user");
  Host.outputString(`Hello, ${user}!`);
}

module.exports = { greet };
```

To test it, the [Extism CLI](https://github.com/extism/cli) has a `--config`
option that lets you pass in `key=value` pairs:

```bash
extism call plugin.wasm greet --config user=Benjamin --wasi
# => Hello, Benjamin!
```

### Variables

Variables are another key-value mechanism but it's a mutable data store that
will persist across function calls. These variables will persist as long as the
host has loaded and not freed the plug-in. You can use `Var.getBytes`,
`Var.getString`, and `Var.set` to manipulate vars:

```javascript
function count() {
  let count = Var.getString("count") || "0";
  count = parseInt(count, 10);
  count += 1;
  Var.set("count", count.toString());
  Host.outputString(count.toString());
}

module.exports = { count };
```

### Logging

At the current time, calling `console.log` emits an `info` log. Please file an
issue or PR if you want to expose the raw logging interface:

```javascript
function logStuff() {
  console.log("Hello, World!");
}

module.exports = { logStuff };
```

Running it, you need to pass a log-level flag:

```
extism call plugin.wasm logStuff --wasi --log-level=info
# => 2023/10/17 14:25:00 Hello, World!
```

### HTTP

HTTP calls can be made using the synchronous API `Http.request`:

```javascript
function callHttp() {
  const request = {
    method: "GET",
    url: "https://jsonplaceholder.typicode.com/todos/1",
  };
  const response = Http.request(request);
  if (response.status != 200) {
    throw new Error(`Got non 200 response ${response.status}`);
  }
  Host.outputString(response.body);
}

module.exports = { callHttp };
```

### Host Functions

Until the js-pdk hits 1.0, we may make changes to this API. To use host
functions you need to declare a TypeScript interface `extism:host/user`:

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

**Note:** These functions may only use `I64` arguments, up to 5 arguments.

To use these you need to use `Host.getFunctions()`:

```typescript
const { myHostFunction1, myHostFunction2 } = Host.getFunctions();
```

Calling them is a similar process to other PDKs. You need to manage the memory
with the Memory object and pass across an offset as the `I64` ptr. Using the
return value means dereferencing the returned `I64` ptr from Memory.

```typescript
function greet() {
  let msg = "Hello from js 1";
  let mem = Memory.fromString(msg);
  let offset = myHostFunction1(mem.offset);
  let response = Memory.find(offset).readString();
  if (response != "myHostFunction1: " + msg) {
    throw Error(`wrong message came back from myHostFunction1: ${response}`);
  }

  msg = { hello: "world!" };
  mem = Memory.fromJsonObject(msg);
  offset = myHostFunction2(mem.offset);
  response = Memory.find(offset).readJsonObject();
  if (response.hello != "myHostFunction2") {
    throw Error(`wrong message came back from myHostFunction2: ${response}`);
  }

  Host.outputString(`Hello, World!`);
}

module.exports = { greet };
```

**IMPORTANT:** Currently, a limitation in the js-pdk is that host functions may
only have up to 5 arguments.

## Using with a bundler

The compiler cli and core engine can now run bundled code. You will want to use
a bundler if you want to want to or include modules from NPM, or write the
plugin in Typescript, for example.

There are 2 primary constraints to using a bundler:

1. Your compiled output must be CJS format, not ESM
2. You must target es2020 or lower

### Using with esbuild

The easiest way to set this up would be to use esbuild. The following is a
quickstart guide to setting up a project:

```bash
# Make a new JS project
mkdir extism-plugin
cd extism-plugin
npm init -y
npm install esbuild @extism/js-pdk --save-dev
mkdir src
mkdir dist
```

Optionally add a `jsconfig.json` or `tsconfig.json` to improve intellisense:

```jsonc
{
  "compilerOptions": {
    "lib": [], // this ensures unsupported globals aren't suggested
    "types": ["@extism/js-pdk"], // while this makes the IDE aware of the ones that are
    "noEmit": true // this is only relevant for tsconfig.json
  },
  "include": ["src/**/*"]
}
```

Add `esbuild.js`:

```js
const esbuild = require("esbuild");
// include this if you need some node support:
// npm i @esbuild-plugins/node-modules-polyfill --save-dev
// const { NodeModulesPolyfillPlugin } = require('@esbuild-plugins/node-modules-polyfill')

esbuild
  .build({
    // supports other types like js or ts
    entryPoints: ["src/index.js"],
    outdir: "dist",
    bundle: true,
    sourcemap: true,
    //plugins: [NodeModulesPolyfillPlugin()], // include this if you need some node support
    minify: false, // might want to use true for production build
    format: "cjs", // needs to be CJS for now
    target: ["es2020"], // don't go over es2020 because quickjs doesn't support it
  });
```

Add a `build` script to your `package.json`:

```json
{
  "name": "extism-plugin",
  // ...
  "scripts": {
    // ...
    "build": "node esbuild.js && extism-js dist/index.js -i src/index.d.ts -o dist/plugin.wasm"
  }
  // ...
}
```

Let's import a module from NPM:

```bash
npm install --save fastest-levenshtein
```

Now make some code in `src/index.js`. You can use `import` to load node_modules:

> **Note**: This module uses the ESM Module syntax. The bundler will transform
> all the code to CJS for us

```js
import { closest, distance } from "fastest-levenshtein";

// this function is private to the module
function privateFunc() {
  return "world";
}

// use any export syntax to export a function be callable by the extism host
export function get_closest() {
  let input = Host.inputString();
  let result = closest(input, ["slow", "faster", "fastest"]);
  Host.outputString(result + " " + privateFunc());
}
```

And a d.ts file for it at `src/index.d.ts`:

```typescript
declare module "main" {
  // Extism exports take no params and return an I32
  export function get_closest(): I32;
}
```

```bash
# Run the build script and the plugin will be compiled to dist/plugin.wasm
npm run build
# You can now call from the extism cli or a host SDK
extism call dist/plugin.wasm get_closest --input="fest" --wasi
# => faster World
```

## Using with React and JSX / TSX

Oftentimes people want their JS plug-ins to control or create views. React and
JSX are a great way to do this. Here is the simplest example. Let's just render
a simple view in a typescript plugin.

First declare a `render` export:

```typescript
declare module "main" {
  export function render(): I32;
}
```

Now install the deps:

```bash
npm install react-dom --save
npm install @types/react --save-dev
```

Now we can make an index.tsx:

```typescript
import { renderToString } from "react-dom/server";
import React from "react";

interface AppProps {
  name: string;
}

function App(props: AppProps) {
  return (
    <>
      <p>Hello ${props.name}!</p>
    </>
  );
}

export function render() {
  const props = JSON.parse(Host.inputString()) as AppProps;
  const app = <App {...props} />;
  Host.outputString(renderToString(app));
}
```

To see a more complex example of how you might build something real, see
[examples/react](./examples/react/)

## Generating Bindings

It's often very useful to define a schema to describe the function signatures
and types you want to use between Extism SDK and PDK languages.

[XTP Bindgen](https://github.com/dylibso/xtp-bindgen) is an open source
framework to generate PDK bindings for Extism plug-ins. It's used by the
[XTP Platform](https://www.getxtp.com/), but can be used outside of the platform
to define any Extism compatible plug-in system.

### 1. Install the `xtp` CLI.

See installation instructions
[here](https://docs.xtp.dylibso.com/docs/cli#installation).

### 2. Create a schema using our OpenAPI-inspired IDL:

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
# components.schemas defined in example-schema.yaml...
```

> See an example in [example-schema.yaml](./example-schema.yaml), or a full
> "kitchen sink" example on
> [the docs page](https://docs.xtp.dylibso.com/docs/concepts/xtp-schema/).

### 3. Generate bindings to use from your plugins:

```
xtp plugin init --schema-file ./example-schema.yaml
  > 1. TypeScript                      
    2. Go                              
    3. Rust                            
    4. Python                          
    5. C#                              
    6. Zig                             
    7. C++                             
    8. GitHub Template                 
    9. Local Template
```

This will create an entire boilerplate plugin project for you to get started
with:

```typescript
/**
 * @returns {VowelReport} The result of counting vowels on the Vowels input.
 */
export function CountVowelsImpl(input: string): VowelReport {
  // TODO: fill out your implementation here
  throw new Error("Function not implemented.");
}
```

Implement the empty function(s), and run `xtp plugin build` to compile your
plugin.

> For more information about XTP Bindgen, see the
> [dylibso/xtp-bindgen](https://github.com/dylibso/xtp-bindgen) repository and
> the official
> [XTP Schema documentation](https://docs.xtp.dylibso.com/docs/concepts/xtp-schema).

## Compiling the compiler from source

### Prerequisites

Before compiling the compiler, you need to install prerequisites.

1. Install Rust using [rustup](https://rustup.rs)
2. Install the WASI target platform via
   `rustup target add --toolchain stable wasm32-wasip1`
3. Install the wasi sdk using the makefile command: `make download-wasi-sdk`
4. Install [CMake](https://cmake.org/install/) (on macOS with homebrew,
   `brew install cmake`)
5. Install [Binaryen](https://github.com/WebAssembly/binaryen/) and add it's
   install location to your PATH (only wasm-opt is required for build process)
6. Install [7zip](https://www.7-zip.org/)(only for Windows)

### Compiling from source

Run make to compile the core crate (the engine) and the cli:

```
make
```

To test the built compiler (ensure you have Extism installed):

```bash
./target/release/extism-js bundle.js -i bundle.d.ts -o out.wasm
extism call out.wasm count_vowels --wasi --input='Hello World Test!'
# => "{\"count\":4}"
```

## How it works

This works a little differently than other PDKs. You cannot compile JS to Wasm
because it doesn't have an appropriate type system to do this. Something like
[Assemblyscript](https://github.com/extism/assemblyscript-pdk) is better suited
for this. Instead, we have compiled QuickJS to Wasm. The `extism-js` command we
have provided here is a little compiler / wrapper that does a series of things
for you:

1. It loads an "engine" Wasm program containing the QuickJS runtime
2. It initializes a QuickJS context
3. It loads your js source code into memory
4. It parses the js source code for exports and generates 1-to-1 proxy export
   functions in Wasm
5. It freezes and emits the machine state as a new Wasm file at this
   post-initialized point in time

This new Wasm file can be used just like any other Extism plugin.

## Why not use Javy?

Javy, and many other high level language Wasm tools, assume use of the _command
pattern_. This is when the Wasm module only exports a main function and
communicates with the host through stdin and stdout. With Extism, we have more
of a shared library interface. The module exposes multiple entry points through
exported functions. Furthermore, Javy has many Javy and Shopify specific things
it's doing that we will not need. However, the core idea is the same, and we can
possibly contribute by adding support to Javy for non-command-pattern modules.
Then separating the Extism PDK specific stuff into another repo.
