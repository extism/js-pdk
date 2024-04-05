import "core-js/actual/url";
import "core-js/actual/url/to-json";
import "core-js/actual/url-search-params";
import { URLPattern } from "urlpattern-polyfill";


declare module globalThis {
  var URLPattern;
  function __decodeUtf8BufferToString(...args): string;
  function __encodeStringToUtf8Buffer(...args): ArrayBufferLike;
  var __getTime;
  var Date;
  var TextDecoder;
  var TextEncoder;
  var MemoryHandle;
  var __Host;
  var __Http;
};

interface MemoryData {
  offset: number,
  len: number,
};

globalThis.URLPattern = URLPattern;

const __decodeUtf8BufferToString = globalThis.__decodeUtf8BufferToString;
const __encodeStringToUtf8Buffer = globalThis.__encodeStringToUtf8Buffer;
const __getTime = globalThis.__getTime;
const __Host = globalThis.__Host;
const __Http = globalThis.__Http;

class __ExtismDate extends Date {
  constructor(arg) {
    if (arg) {
      super(arg);
    } else {
      super(__getTime());
    }
  }
}

globalThis.Date = __ExtismDate;

interface TextDecoderOptions {
  ignoreBOM?: boolean;
  fatal?: boolean; 
}

interface DecodeOptions {
  stream?: any, 
};

class TextDecoder {
  constructor(label: string = "utf-8", options: TextDecoderOptions = {}) {
    label = label.trim().toLowerCase();
    const acceptedLabels = [
      "utf-8",
      "utf8",
      "unicode-1-1-utf-8",
      "unicode11utf8",
      "unicode20utf8",
      "x-unicode20utf8",
    ];
    if (!acceptedLabels.includes(label)) {
      // Not spec-compliant behaviour
      throw new RangeError("The encoding label provided must be utf-8");
    }
    Object.defineProperties(this, {
      encoding: { value: "utf-8", enumerable: true, writable: false },
      fatal: { value: !!options.fatal, enumerable: true, writable: false },
      ignoreBOM: {
        value: !!options.ignoreBOM,
        enumerable: true,
        writable: false,
      },
    });
  }

  decode(input, options: DecodeOptions = {}) {
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
      throw new TypeError(
        "The provided value is not of type '(ArrayBuffer or ArrayBufferView)'",
      );
    }

    return __decodeUtf8BufferToString(
      input,
      byteOffset,
      byteLength,
      //@ts-ignore
      this.fatal,
      //@ts-ignore
      this.ignoreBOM,
    );
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

export class MemoryHandle {
  offset: number;
  len: number;

  constructor(offset: number, len: number) {
    this.offset = offset;
    this.len = len;
  }

  readString() {
    return new TextDecoder().decode(this.readBytes());
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

  readUFloat64() {
    const bytes = this.readBytes();
    const arr = new Float64Array(bytes);
    return arr[0];
  }

  readBytes() {
    // @ts-ignore
    return __Memory._readBytes(this.offset);
  }

  readJsonObject() {
    return JSON.parse(this.readString());
  }

  free() {
    // @ts-ignore
    __Memory._free(this.offset);
  }
}

globalThis.TextDecoder = TextDecoder;
globalThis.TextEncoder = TextEncoder;
globalThis.MemoryHandle = MemoryHandle;

export class Memory {
  public static fromString(str: string): MemoryHandle {
    // todo validate
    let bytes = new TextEncoder().encode(str).buffer;
    // @ts-ignore
    const memData = __Memory._fromBuffer(bytes);
    return new MemoryHandle(memData.offset, memData.len);
  };

  public static fromBuffer(bytes: ArrayBufferLike): MemoryHandle {
    // todo validate
    // @ts-ignore
    const memData = __Memory._fromBuffer(bytes);
    return new MemoryHandle(memData.offset, memData.len);
  };

  public static fromJsonObject(obj: JSON): MemoryHandle {
    // todo validate
    const memData = Memory.fromString(JSON.stringify(obj));
    return new MemoryHandle(memData.offset, memData.len);
  };

  public static allocUInt32(i: number): MemoryHandle {
    const buffer = new ArrayBuffer(4);
    const arr = new Uint32Array(buffer);
    arr[0] = i;
    return this.fromBuffer(buffer);
  };

  public static allocUInt64(i: bigint): MemoryHandle {
    const buffer = new ArrayBuffer(8);
    const arr = new BigUint64Array(buffer);
    arr[0] = i;
    return this.fromBuffer(buffer);
  };

  public static allocFloat32(i: number): MemoryHandle {
    const buffer = new ArrayBuffer(4);
    const arr = new Float32Array(buffer);
    arr[0] = i;
    return this.fromBuffer(buffer);
  };

  public static allocFloat64(i: number): MemoryHandle {
    const buffer = new ArrayBuffer(8);
    const arr = new Float64Array(buffer);
    arr[0] = i;
    return this.fromBuffer(buffer);
  };

  public static find(offset: number): MemoryHandle {
    // todo validate
    // @ts-ignore
    const memData = __Memory._find(offset);
    return new MemoryHandle(memData.offset, memData.len);
  };
}

export class Host {
  public static getFunctions(): Object {
    const funcs = {};
    let funcIdx = 0;
    const createInvoke = (funcIdx: number, results: number) => {
      return (...args: any[]) => {
        if (results == 0) {
          // @ts-ignore
          return __Host.invokeFunc0(funcIdx, ...args);
        } else {
          // @ts-ignore
          return __Host.invokeFunc(funcIdx, ...args);
        }
      };
    };

    // @ts-ignore
    __Host.__hostFunctions.forEach((x) => {
      funcs[x.name] = createInvoke(funcIdx++, x.results);
    });
    return funcs;
  };                                          
  // @ts-ignore  
  public static inputBytes(): ArrayBufferLike { return __Host.inputBytes() };
  // @ts-ignore  
  public static inputString(): string { return __Host.inputString() }; 
  // @ts-ignore  
  public static outputBytes(output: ArrayBufferLike) { __Host.outputBytes(output) };
  // @ts-ignore  
  public static outputString(output: string) { __Host.outputString(output) };
}

export interface HttpRequest {
  url: string;
  method?: "GET" | "HEAD" | "POST" | "PUT" | "DELETE" | "CONNECT" | "OPTIONS" | "TRACE" | "PATCH";
  headers?: {};
}

export interface HttpResponse {
  body: string;
  status: number;
}

export class Http {
  public static request(req: HttpRequest, body?: ArrayBufferLike): HttpResponse {
    if (body) {
      const s = new Uint8Array(body).toString()
      // @ts-ignore
      return __Http.request(req, s);
    } 

    // @ts-ignore
    return __Http.request(req);
  }
}

export class Var {
  public static set(name: string, value: string | ArrayBufferLike) {
    // @ts-ignore
    __Var.set(name, value);
  }

  public static getBytes(name: string): ArrayBufferLike | null {
    // @ts-ignore
    return __Var.getBytes(name);
  }

  public static getString(name: string): string | null {
    // @ts-ignore
    return __Var.getString(name);
  }
}

export class Config {
  public static get(key: string): string | null {
    // @ts-ignore
    return __Config.get(key)
  }
}