set -e

OS=''
case `uname` in
  Darwin*)  OS="macos" ;;
  Linux*)   OS="linux" ;;
  *)        echo "unknown os: $OSTYPE" && exit 1 ;;
esac

ARCH=`uname -m`
case "$ARCH" in
  ix86*|x86_64*)    ARCH="x86_64" ;;
  arm64*|aarch64*)  ARCH="aarch64" ;;
  *)                echo "unknown arch: $ARCH" && exit 1 ;;
esac


export TAG="v1.0.0-rc4"
export BINARYEN_TAG="version_116"


curl -L -O "https://github.com/extism/js-pdk/releases/download/$TAG/extism-js-$ARCH-$OS-$TAG.gz"
curl -L -O "https://github.com/WebAssembly/binaryen/releases/download/$BINARYEN_TAG/binaryen-$BINARYEN_TAG-$ARCH-$OS.tar.gz"

gunzip extism-js*.gz
sudo mv extism-js-* /usr/local/bin/extism-js
chmod +x /usr/local/bin/extism-js

tar xvf "binaryen-$BINARYEN_TAG-$ARCH-$OS.tar.gz"
mv "binaryen-$BINARYEN_TAG"/ binaryen/
sudo mkdir /usr/local/binaryen
sudo mv binaryen/bin/wasm* /usr/local/binaryen

for file in $(ls /usr/local/binaryen); do
  sudo ln -s  /usr/local/binaryen/$file /usr/local/bin/$file
done
