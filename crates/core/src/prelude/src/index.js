import 'core-js/actual/url';
import 'core-js/actual/url/to-json';
import 'core-js/actual/url-search-params';
import { URLPattern } from 'urlpattern-polyfill';

globalThis.URLPattern = URLPattern;

const __decodeUtf8BufferToString = globalThis.__decodeUtf8BufferToString;
const __encodeStringToUtf8Buffer = globalThis.__encodeStringToUtf8Buffer;

class TextDecoder {
  constructor(label = "utf-8", options = {}) {
    label = label.trim().toLowerCase();
    const acceptedLabels = ["utf-8", "utf8", "unicode-1-1-utf-8", "unicode11utf8", "unicode20utf8", "x-unicode20utf8"];
    if (!acceptedLabels.includes(label)) {
      // Not spec-compliant behaviour
      throw new RangeError("The encoding label provided must be utf-8");
    }
    Object.defineProperties(this, {
      encoding: { value: "utf-8", enumerable: true, writable: false },
      fatal: { value: !!options.fatal, enumerable: true, writable: false },
      ignoreBOM: { value: !!options.ignoreBOM, enumerable: true, writable: false },
    })
  }

  decode(input, options = {}) {
    if (input === undefined) {
      return "";
    }

    if (options.stream) {
      throw new Error("Streaming decode is not supported");
    }

    // backing buffer would not have byteOffset and may have different byteLength
    let byteOffset = input.byteOffset || 0;
    let byteLength = input.byteLength;
    if (ArrayBuffer.isView(input)) {
      input = input.buffer;
    }

    if (!(input instanceof ArrayBuffer)) {
      throw new TypeError("The provided value is not of type '(ArrayBuffer or ArrayBufferView)'");
    }

    return __decodeUtf8BufferToString(input, byteOffset, byteLength, this.fatal, this.ignoreBOM);
  }
}

class TextEncoder {
  constructor() {
    Object.defineProperties(this, {
      encoding: { value: "utf-8", enumerable: true, writable: false },
    });
  }

  encode(input = "") {
    input = input.toString(); // non-string inputs are converted to strings
    return new Uint8Array(__encodeStringToUtf8Buffer(input));
  }

  encodeInto(source, destination) {
    throw new Error("encodeInto is not supported");
  }
}

globalThis.TextDecoder = TextDecoder;
globalThis.TextEncoder = TextEncoder;

Host.getFunctions = () => {
  const funcs = {}
  let funcIdx = 0
  const createInvoke = (funcIdx) => {
    return (ptr) => {
      console.log(`name and func ${funcIdx} ptr ${ptr}`)
      return Host.invokeFunc(funcIdx, ptr)
    }
  }
  Host.__hostFunctions.forEach(name => {
    funcs[name] = createInvoke(funcIdx++)
  })
  return funcs
}
