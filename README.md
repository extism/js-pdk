# Extism JavaScript PDK

**Note**: This is very experimental. If you are interested in helping or following development, join the [#js-pdk](https://discord.com/channels/1011124058408112148/1062468347851178165) room in our discord channel.

## Overview

This PDK uses [quickjs](https://bellard.org/quickjs/) and [wizer](https://github.com/bytecodealliance/wizer) to run javascript as an Extism Plug-in.

This is essentially a fork of [javy](https://github.com/Shopify/javy) by Shopify. We may wish to collaborate and upstream some things to them. For the time being I built this up from scratch using some of their crates, namely quickjs-wasm-rs.

## Why not use Javy?

Javy, and many other high level language Wasm tools, assume use of the *command pattern*. This is when the Wasm module only exports a main function and communicates with the host through stdin and stdout. With Extism, we have more of a library approach. The module exposes multiple entry points through exported functions. Furthermore, Javy has many Javy and Shopify specific things it's doing that we will not need. However the core idea is the same and we can possibly contribute by adding support to Javy for non-command-pattern modules. Then separating the Extism PDK specific stuff into another repo.

## Install

We now have released binaries. Check the [releases](/releases) page for the latest. I can't give windows instructions yet but this should work for mac and linux:

```bash
export TAG=v0.1.1
wget "https://github.com/extism/js-pdk/releases/download/$TAG/extism-js-x86_64-macos-$TAG.gz"
gunzip extism-js*.gz
sudo mv extism-js-* /usr/local/bin/extism-js
chmod +x /usr/local/bin/extism-js
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

> **Note**: If you are using mac you may need to tell your security system this unsigned binary is fine. If you think this is dangerous, or can't get it to work, see the compile from source section below.

Try it on a script (script.js):

```javascript
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
    return 0
}

module.exports = { count_vowels };
```

```bash
extism-js script.js -o count_vowels.wasm
extism call count_vowels.wasm count_vowels --input="Hello World!" --wasi
# => {"count":3}                          
```

## Exports in JS

For the most part, you can write your JS how you want. In order to export a function to Extism you must use this `module.exports = {f1, f2, ..}` syntax:

```js

func myFunc() {
    //...
}

module.exports = { myFunc }
```

We will better support module systems in the future but for now we are hard coding the compiler to this one export syntax.

## Compiling the compiler from source

You need to compile from source at the moment. Here are the instructions:

You need the wasi sdk which can be fetched with the makefile:

```
make download-wasi-sdk
```

Then run make to compile the core crate (the engine) and the cli:

```
make
```

You now have a CLI that will allow you to create an Extism compatible Wasm program from a JS file. There is one in the root of this directory called [script.js](/script.js).


```
./target/release/extism-js script.js -o out.wasm
```

You can now use this in Extism:

```
extism call out.wasm count_vowels --wasi --input="Hello World Test!"

"{\"count\":4}"
```


## What needs to be done?

I've got the exports to work, but it's a fragile and complicated solution. Will write it up soon, and maybe it can be replaced with something simpler.

We need to finish the binding of the Extism PDK functions to the JS world. This is not too hard with quickjs-wasm-rs. We should mostly use all the code already written in rust and just map to JS types. 
