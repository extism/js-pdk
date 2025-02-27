#!/usr/bin/env bash

#set -euo pipefail

if [[ "$(basename $(pwd))" != "js-pdk" ]]; then
    echo "Run this inside in the root of the js-pdk repo" 1>&2
    exit 1
fi

# Don't try and install the wasi-sdk if the user has specified the wasi-sdk is installed elsewhere
set +u
if [[ -n "$QUICKJS_WASM_SYS_WASI_SDK_PATH" ]]; then
    # Check that something is present where the user says the wasi-sdk is located
    if [[ ! -d "$QUICKJS_WASM_SYS_WASI_SDK_PATH" ]]; then
        echo "Download the wasi-sdk to $QUICKJS_WASM_SYS_WASI_SDK_PATH" 1>&2
        exit 1
    fi
    exit 0
fi
set -u

ARCH=`uname -m`
case "$ARCH" in
  ix86*|x86_64*)    ARCH="x86_64" ;;
  arm64*|aarch64*)  ARCH="arm64" ;;
  *)                echo "unknown arch: $ARCH" && exit 1 ;;
esac

PATH_TO_SDK="wasi-sdk"
if [[ ! -d $PATH_TO_SDK ]]; then
    TMPGZ=$(mktemp)
    VERSION_MAJOR="24"
    VERSION_MINOR="0"
    if [[ "$(uname -s)" == "Darwin" ]]; then
        curl --fail --location https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-${VERSION_MAJOR}/wasi-sdk-${VERSION_MAJOR}.${VERSION_MINOR}-${ARCH}-macos.tar.gz --output $TMPGZ
    else
        curl --fail --location https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-${VERSION_MAJOR}/wasi-sdk-${VERSION_MAJOR}.${VERSION_MINOR}-${ARCH}-linux.tar.gz --output $TMPGZ
    fi
    mkdir $PATH_TO_SDK
    tar xf $TMPGZ -C $PATH_TO_SDK --strip-components=1
fi
