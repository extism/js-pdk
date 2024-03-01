import 'core-js/actual/url';
import 'core-js/actual/url/to-json';
import 'core-js/actual/url-search-params';
import { URLPattern } from 'urlpattern-polyfill';

globalThis.URLPattern = URLPattern;

const __decodeUtf8BufferToString = globalThis.__decodeUtf8BufferToString;
const __encodeStringToUtf8Buffer = globalThis.__encodeStringToUtf8Buffer;
const __getTime = globalThis.__getTime

class __ExtismDate extends Date {
  constructor(arg) {
    if (arg) {
      super(arg)
    } else {
      super(__getTime())
    }
  }
}

globalThis.Date = __ExtismDate

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

class MemoryHandle {
  constructor(offset, len) {
    this.offset = offset
    this.len = len
  }

  readString() {
    return new TextDecoder().decode(this.readBytes())
  }

  readUInt32() {
    const bytes = this.readBytes();
    const arr = new Uint32Array(bytes);
    return arr[0];
  }

  readUInt64() {
    const bytes = this.readBytes();
    const arr = new BigUint64Array(bytes);
    return arr[0];
  }

  readFloat32() {
    const bytes = this.readBytes();
    const arr = new Float32Array(bytes);
    return arr[0];
  }

  readUInt64() {
    const bytes = this.readBytes();
    const arr = new Float64Array(bytes);
    return arr[0];
  }

  readBytes() {
    return Memory._readBytes(this.offset)
  }

  readJsonObject() {
    return JSON.parse(this.readString())
  }
}

globalThis.TextDecoder = TextDecoder;
globalThis.TextEncoder = TextEncoder;
globalThis.MemoryHandle = MemoryHandle;

Memory.fromString = (str) => {
  // todo validate
  let bytes = new TextEncoder().encode(str).buffer
  const memData = Memory._fromBuffer(bytes)
  return new MemoryHandle(memData.offset, memData.len)
}

Memory.fromBuffer = (bytes) => {
  // todo validate
  const memData = Memory._fromBuffer(bytes)
  return new MemoryHandle(memData.offset, memData.len)
}

Memory.fromJsonObject = (obj) => {
  // todo validate
  const memData = Memory.fromString(JSON.stringify(obj))
  return new MemoryHandle(memData.offset, memData.len)
}

Memory.allocUInt32 = (i) => {
  const buffer = new ArrayBuffer(4);
  const arr = new Uint32Array(buffer);
  arr[0] = i;
  return Memory.fromBuffer(buffer);
}

Memory.allocUInt64 = (i) => {
  const buffer = new ArrayBuffer(8);
  const arr = new BigUint64Array(buffer);
  arr[0] = i;
  return Memory.fromBuffer(buffer);
}

Memory.allocFloat32 = (i) => {
  const buffer = new ArrayBuffer(4);
  const arr = new Float32Array(buffer);
  arr[0] = i;
  return Memory.fromBuffer(buffer);
}

Memory.allocFloat64 = (i) => {
  const buffer = new ArrayBuffer(8);
  const arr = new Float64Array(buffer);
  arr[0] = i;
  return Memory.fromBuffer(buffer);
}

Memory.find = (offset) => {
  // todo validate
  const memData = Memory._find(offset)
  return new MemoryHandle(memData.offset, memData.len)
}

Host.getFunctions = () => {
  const funcs = {}
  let funcIdx = 0
  const createInvoke = (funcIdx, results) => {
    return (...args) => {
      if (results == 0) {
        return Host.invokeFunc0(funcIdx, ...args)
      } else {
        return Host.invokeFunc(funcIdx, ...args)
      }
    }
  }
  Host.__hostFunctions.forEach((x) => {
    funcs[x.name] = createInvoke(funcIdx++, x.results)
  })
  return funcs
}
