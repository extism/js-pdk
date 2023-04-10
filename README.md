# Extism JavaScript PDK

> **Note**: This is very experimental. If you are interested in helping or following development, join the [#js-pdk](https://discord.gg/ZACPSVz9) room in our discord channel.

## Overview

This PDK uses [QuickJS](https://bellard.org/quickjs/) and [wizer](https://github.com/bytecodealliance/wizer) to run javascript as an Extism Plug-in.

This is essentially a fork of [Javy](https://github.com/Shopify/javy) by Shopify. We may wish to collaborate and upstream some things to them. For the time being I built this up from scratch using some of their crates, namely quickjs-wasm-rs.

## How it works

This works a little differently than other PDKs. You cannot compile JS to Wasm because it doesn't have an appropriate type system to do this. Something like [Assemblyscript](https://www.assemblyscript.org/) is better suited for this. Instead, we have compiled QuickJS to Wasm. The `extism-js` command we have provided here is a little compiler / wrapper that does a series of things for you:

1. It loads an "engine" Wasm program containing the QuickJS runtime
2. It initializes a QuickJS context
3. It loads your js source code into memory
4. It parses the js source code for exports and generates 1-to-1 proxy export functions in Wasm
5. It freezes and emits the machine state as a new Wasm file at this post-initialized point in time

This new Wasm file can be used just like any other Extism plugin.

## Install the compiler

We now have released binaries. Check the [releases](https://github.com/extism/js-pdk/releases) page for the latest.

> **Note**: Windows is not currently a supported platform, only mac and linux

## Install Script

```bash
curl -O https://raw.githubusercontent.com/extism/js-pdk/main/install.sh
sh install.sh
```

Then run command with no args to see the help:

```
extism-js
error: The following required arguments were not provided:
    <input>

USAGE:
    extism-js <input> -o <output>

For more information try --help
```

> **Note**: If you are using mac, you may need to tell your security system this unsigned binary is fine. If you think this is dangerous, or can't get it to work, see the "compile from source" section below.

Try it on a script file. Name this `script.js:

> **Note**: You must use [CJS Module syntax](https://nodejs.org/api/modules.html#modules-commonjs-modules) when not using a bundler.

```javascript
// script.js

const VOWELS = [
    'a', 'e', 'i', 'o', 'u',
]

function count_vowels() {
    let input = Host.inputString()
    let count = 0
    for (let i = 0; i < input.length; i++) {
        if (VOWELS.includes(input[i].toLowerCase())) {
            count += 1
        }
    }
    Host.outputString(JSON.stringify({count}))
}

module.exports = {count_vowels}
```

```bash
extism-js script.js -o count_vowels.wasm
extism call count_vowels.wasm count_vowels --input="Hello World!" --wasi
# => {"count":3}                          
```

## Using with a bundler

The compiler cli and core engine can now run bundled code. You will want to use a bundler
if you want to want to or include modules from NPM, or write the plugin in Typescript, for example.

There are 2 primary constraints to using a bundler:

1. Your compiled output must be CJS format, not ESM
2. You must target es2020 or lower


### Using with esbuild

The easiest way to set this up would be to use esbuild. The following is a quickstart guide to setting up a project:

```bash
# Make a new JS project
mkdir extism-plugin
cd extism-plugin
npm init -y
npm install esbuild --save-dev
mkdir src
mkdir dist
```


Add `esbuild.js`:

```js
const esbuild = require('esbuild');

esbuild
    .build({
        entryPoints: ['src/index.js'],
        outdir: 'dist',
        bundle: true,
        sourcemap: true,
        minify: false, // might want to use true for production build
        format: 'cjs', // needs to be CJS for now
        target: ['es2020'] // don't go over es2020 because quickjs doesn't support it
    })
```

Add a `build` script to your `package.json`:

```json
{
  "name": "extism-plugin",
  // ...
  "scripts": {
    // ...
    "build": "node esbuild.js && extism-js dist/index.js -o dist/plugin.wasm"
  },
  // ...
}
```

Let's import a module from NPM:

```bash
npm install --save fastest-levenshtein
```

Now make some code in `src/index.js`. You can use `import` to load node_modules:

> **Note**: This module uses the ESM Module syntax. The bundler will transform all the code to CJS for us

```js
import {distance, closest} from 'fastest-levenshtein'

// this function is private to the module
function privateFunc() { return 'world' }

// use any export syntax to export a function be callable by the extism host
export function get_closest() {
  let input = Host.inputString()
  let result = closest(input, ['slow', 'faster', 'fastest'])
  Host.outputString(result + ' ' + privateFunc())
}
```

```bash
# Run the build script and the plugin will be compiled to dist/plugin.wasm
npm run build
# You can now call from the extism cli or a host SDK
extism call dist/plugin.wasm get_closest --input="fest" --wasi
faster World
```

## Compiling the compiler from source

### Prerequisites
Before compiling the compiler, you need to install prerequisites.

1. Install Rust using [rustup](https://rustup.rs)
2. Install the WASI target platform via `rustup target add --toolchain stable wasm32-wasi`
3. Install the wasi sdk using the makefile command: `make download-wasi-sdk`
4. Install [CMake](https://cmake.org/install/) (on macOS with homebrew, `brew install cmake`)


### Compiling from source

Run make to compile the core crate (the engine) and the cli:

```
make
```

To test the built compiler (ensure you have Extism installed):
```bash
./target/release/extism-js bundle.js -o out.wasm
extism call out.wasm count_vowels --wasi --input='Hello World Test!'
# => "{\"count\":4}"
```

## Why not use Javy?

Javy, and many other high level language Wasm tools, assume use of the *command pattern*. This is when the Wasm module only exports a main function and communicates with the host through stdin and stdout. With Extism, we have more of a shared library interface. The module exposes multiple entry points through exported functions. Furthermore, Javy has many Javy and Shopify specific things it's doing that we will not need. However, the core idea is the same, and we can possibly contribute by adding support to Javy for non-command-pattern modules. Then separating the Extism PDK specific stuff into another repo.

## What needs to be done?

Implemented so far:

* Host.inputBytes
* Host.inputString
* Host.outputBytes
* Host.outputString
* Var.getBytes
* Var.getString
* Var.set
* Config.get
* Http.request
* console.log
* console.error
* throw Error

The above are implemented but need some more validation and resilience built into them. debating whether I should implement the bulk of the code in js or rust. Working on implementing the other pdk methods.

I've got the exports to work, but it's a fragile and complicated solution. Will write it up soon, and maybe it can be replaced with something simpler.
