/*! *****************************************************************************
Copyright (c) Microsoft Corporation. All rights reserved.
Licensed under the Apache License, Version 2.0 (the "License"); you may not use
this file except in compliance with the License. You may obtain a copy of the
License at http://www.apache.org/licenses/LICENSE-2.0

THIS CODE IS PROVIDED ON AN *AS IS* BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
KIND, EITHER EXPRESS OR IMPLIED, INCLUDING WITHOUT LIMITATION ANY IMPLIED
WARRANTIES OR CONDITIONS OF TITLE, FITNESS FOR A PARTICULAR PURPOSE,
MERCHANTABLITY OR NON-INFRINGEMENT.

See the Apache Version 2.0 License for specific language governing permissions
and limitations under the License.
***************************************************************************** */

/// <reference types="urlpattern-polyfill" />

/**
 * The URL interface represents an object providing static methods used for creating object URLs.
 *
 * [MDN Reference](https://developer.mozilla.org/docs/Web/API/URL)
 */
interface URL {
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/URL/hash) */
  hash: string;
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/URL/host) */
  host: string;
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/URL/hostname) */
  hostname: string;
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/URL/href) */
  href: string;
  toString(): string;
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/URL/origin) */
  readonly origin: string;
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/URL/password) */
  password: string;
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/URL/pathname) */
  pathname: string;
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/URL/port) */
  port: string;
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/URL/protocol) */
  protocol: string;
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/URL/search) */
  search: string;
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/URL/searchParams) */
  readonly searchParams: URLSearchParams;
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/URL/username) */
  username: string;
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/URL/toJSON) */
  toJSON(): string;
}

declare var URL: {
  prototype: URL;
  new (url: string | URL, base?: string | URL): URL;
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/URL/canParse_static) */
  canParse(url: string | URL, base?: string): boolean;
};

/** [MDN Reference](https://developer.mozilla.org/docs/Web/API/URLSearchParams) */
interface URLSearchParams {
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/URLSearchParams/size) */
  readonly size: number;
  /**
   * Appends a specified key/value pair as a new search parameter.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/URLSearchParams/append)
   */
  append(name: string, value: string): void;
  /**
   * Deletes the given search parameter, and its associated value, from the list of all search parameters.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/URLSearchParams/delete)
   */
  delete(name: string, value?: string): void;
  /**
   * Returns the first value associated to the given search parameter.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/URLSearchParams/get)
   */
  get(name: string): string | null;
  /**
   * Returns all the values association with a given search parameter.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/URLSearchParams/getAll)
   */
  getAll(name: string): string[];
  /**
   * Returns a Boolean indicating if such a search parameter exists.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/URLSearchParams/has)
   */
  has(name: string, value?: string): boolean;
  /**
   * Sets the value associated to a given search parameter to the given value. If there were several values, delete the others.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/URLSearchParams/set)
   */
  set(name: string, value: string): void;
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/URLSearchParams/sort) */
  sort(): void;
  /** Returns a string containing a query string suitable for use in a URL. Does not include the question mark. */
  toString(): string;
  forEach(
    callbackfn: (value: string, key: string, parent: URLSearchParams) => void,
    thisArg?: any
  ): void;
}

declare var URLSearchParams: {
  prototype: URLSearchParams;
  new (
    init?: string[][] | Record<string, string> | string | URLSearchParams
  ): URLSearchParams;
};

interface TextDecodeOptions {
  stream?: boolean;
}

interface TextDecoderOptions {
  fatal?: boolean;
  ignoreBOM?: boolean;
}

interface TextEncoderEncodeIntoResult {
  read: number;
  written: number;
}

type AllowSharedBufferSource = ArrayBuffer | ArrayBufferView;

/**
 * A decoder for a specific method, that is a specific character encoding, like utf-8, iso-8859-2, koi8, cp1261, gbk, etc. A decoder takes a stream of bytes as input and emits a stream of code points. For a more scalable, non-native library, see StringView – a C-like representation of strings based on typed arrays.
 *
 * [MDN Reference](https://developer.mozilla.org/docs/Web/API/TextDecoder)
 */
interface TextDecoder extends TextDecoderCommon {
  /**
   * Returns the result of running encoding's decoder. The method can be invoked zero or more times with options's stream set to true, and then once without options's stream (or set to false), to process a fragmented input. If the invocation without options's stream (or set to false) has no input, it's clearest to omit both arguments.
   *
   * ```
   * var string = "", decoder = new TextDecoder(encoding), buffer;
   * while(buffer = next_chunk()) {
   *   string += decoder.decode(buffer, {stream:true});
   * }
   * string += decoder.decode(); // end-of-queue
   * ```
   *
   * If the error mode is "fatal" and encoding's decoder returns error, throws a TypeError.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/TextDecoder/decode)
   */
  decode(input?: AllowSharedBufferSource, options?: TextDecodeOptions): string;
}

declare var TextDecoder: {
  prototype: TextDecoder;
  new (label?: string, options?: TextDecoderOptions): TextDecoder;
};

interface TextDecoderCommon {
  /**
   * Returns encoding's name, lowercased.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/TextDecoder/encoding)
   */
  readonly encoding: string;
  /**
   * Returns true if error mode is "fatal", otherwise false.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/TextDecoder/fatal)
   */
  readonly fatal: boolean;
  /**
   * Returns the value of ignore BOM.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/TextDecoder/ignoreBOM)
   */
  readonly ignoreBOM: boolean;
}

/**
 * TextEncoder takes a stream of code points as input and emits a stream of bytes. For a more scalable, non-native library, see StringView – a C-like representation of strings based on typed arrays.
 *
 * [MDN Reference](https://developer.mozilla.org/docs/Web/API/TextEncoder)
 */
interface TextEncoder extends TextEncoderCommon {
  /**
   * Returns the result of running UTF-8's encoder.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/TextEncoder/encode)
   */
  encode(input?: string): Uint8Array;
  /**
   * Runs the UTF-8 encoder on source, stores the result of that operation into destination, and returns the progress made as an object wherein read is the number of converted code units of source and written is the number of bytes modified in destination.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/TextEncoder/encodeInto)
   */
  encodeInto(
    source: string,
    destination: Uint8Array
  ): TextEncoderEncodeIntoResult;
}

declare var TextEncoder: {
  prototype: TextEncoder;
  new (): TextEncoder;
};

interface TextEncoderCommon {
  /**
   * Returns "utf-8".
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/TextEncoder/encoding)
   */
  readonly encoding: string;
}

/** [MDN Reference](https://developer.mozilla.org/docs/Web/API/console) */
interface Console {
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/console/assert_static) */
  assert(condition?: boolean, ...data: any[]): void;
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/console/count_static) */
  count(label?: string): void;
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/console/countReset_static) */
  countReset(label?: string): void;
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/console/debug_static) */
  debug(...data: any[]): void;
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/console/error_static) */
  error(...data: any[]): void;
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/console/info_static) */
  info(...data: any[]): void;
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/console/log_static) */
  log(...data: any[]): void;
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/console/time_static) */
  time(label?: string): void;
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/console/timeEnd_static) */
  timeEnd(label?: string): void;
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/console/timeLog_static) */
  timeLog(label?: string, ...data: any[]): void;
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/console/table_static) */
  table(data: any, columns?: string[]): void;
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/console/trace_static) */
  trace(...data: any[]): void;
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/console/warn_static) */
  warn(...data: any[]): void;
}

declare var console: Console;

/** [MDN Reference](https://developer.mozilla.org/docs/Web/API/atob) */
declare function atob(data: string): string;
/** [MDN Reference](https://developer.mozilla.org/docs/Web/API/btoa) */
declare function btoa(data: string): string;

/** [MDN Reference](https://developer.mozilla.org/docs/Web/API/structuredClone) */
declare function structuredClone<T>(value: T): T;

/** [MDN Reference](https://developer.mozilla.org/docs/Web/API/Performance) */
interface Performance {
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/Performance/now) */
  now(): number;
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/Performance/timeOrigin) */
  readonly timeOrigin: number;
}

declare var performance: Performance;

/** [MDN Reference](https://developer.mozilla.org/docs/Web/API/Headers) */
interface Headers {
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/Headers/append) */
  append(name: string, value: string): void;
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/Headers/delete) */
  delete(name: string): void;
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/Headers/get) */
  get(name: string): string | null;
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/Headers/has) */
  has(name: string): boolean;
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/Headers/set) */
  set(name: string, value: string): void;
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/Headers/forEach) */
  forEach(
    callback: (value: string, key: string, parent: Headers) => void,
  ): void;
  entries(): [string, string][];
  keys(): string[];
  values(): string[];
}

declare var Headers: {
  prototype: Headers;
  new (
    init?: Record<string, string> | [string, string][] | Headers,
  ): Headers;
};

/** [MDN Reference](https://developer.mozilla.org/docs/Web/API/Response) */
interface Response {
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/Response/body) */
  readonly bodyUsed: boolean;
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/Response/headers) */
  readonly headers: Headers;
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/Response/ok) */
  readonly ok: boolean;
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/Response/status) */
  readonly status: number;
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/Response/statusText) */
  readonly statusText: string;
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/Response/url) */
  readonly url: string;
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/Response/arrayBuffer) */
  arrayBuffer(): Promise<ArrayBuffer>;
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/Response/clone) */
  clone(): Response;
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/Response/json) */
  json(): Promise<any>;
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/Response/text) */
  text(): Promise<string>;
}

declare var Response: {
  prototype: Response;
  new (body?: string | null, init?: ResponseInit): Response;
};

interface ResponseInit {
  headers?: Headers | Record<string, string> | [string, string][];
  status?: number;
  statusText?: string;
}

interface RequestInit {
  body?: string;
  headers?: Headers | Record<string, string> | [string, string][];
  method?: string;
}

/** [MDN Reference](https://developer.mozilla.org/docs/Web/API/fetch) */
declare function fetch(
  input: string | URL,
  init?: RequestInit,
): Promise<Response>;

/** [MDN Reference](https://developer.mozilla.org/docs/Web/API/Crypto) */
interface Crypto {
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/Crypto/getRandomValues) */
  getRandomValues<T extends ArrayBufferView>(array: T): T;
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/Crypto/randomUUID) */
  randomUUID(): `${string}-${string}-${string}-${string}-${string}`;
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/Crypto/subtle) */
  readonly subtle: SubtleCrypto;
}

/** [MDN Reference](https://developer.mozilla.org/docs/Web/API/SubtleCrypto) */
interface SubtleCrypto {
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/SubtleCrypto/digest) */
  digest(
    algorithm: string | { name: string },
    data: ArrayBuffer | ArrayBufferView,
  ): Promise<ArrayBuffer>;
}

/** [MDN Reference](https://developer.mozilla.org/docs/Web/API/crypto) */
declare var crypto: Crypto;

/** [MDN Reference](https://developer.mozilla.org/docs/Web/API/DOMException) */
interface DOMException extends Error {
  readonly code: number;
  readonly name: string;
  readonly message: string;
}

declare var DOMException: {
  prototype: DOMException;
  new (message?: string, name?: string): DOMException;
};

/** [MDN Reference](https://developer.mozilla.org/docs/Web/API/Event) */
interface Event {
  readonly type: string;
  readonly target: EventTarget | null;
  readonly currentTarget: EventTarget | null;
  readonly bubbles: boolean;
  readonly cancelable: boolean;
  readonly defaultPrevented: boolean;
  readonly timeStamp: number;
  preventDefault(): void;
  stopPropagation(): void;
  stopImmediatePropagation(): void;
}

declare var Event: {
  prototype: Event;
  new (
    type: string,
    eventInitDict?: { bubbles?: boolean; cancelable?: boolean },
  ): Event;
};

/** [MDN Reference](https://developer.mozilla.org/docs/Web/API/EventTarget) */
interface EventTarget {
  addEventListener(
    type: string,
    callback: Function | null,
    options?: { once?: boolean } | boolean,
  ): void;
  removeEventListener(type: string, callback: Function | null): void;
  dispatchEvent(event: Event): boolean;
}

declare var EventTarget: {
  prototype: EventTarget;
  new (): EventTarget;
};

/** [MDN Reference](https://developer.mozilla.org/docs/Web/API/queueMicrotask) */
declare function queueMicrotask(callback: () => void): void;

/** [MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/self) */
declare var self: typeof globalThis;

/** [Node.js Reference](https://nodejs.org/api/buffer.html) */
declare class Buffer extends Uint8Array {
  static from(
    value: string,
    encoding?: BufferEncoding,
  ): Buffer;
  static from(
    value: ArrayBuffer,
    byteOffset?: number,
    length?: number,
  ): Buffer;
  static from(value: Buffer | Uint8Array | number[]): Buffer;
  static from(value: { type: "Buffer"; data: number[] }): Buffer;
  static alloc(
    size: number,
    fill?: number | string | Buffer,
    encoding?: BufferEncoding,
  ): Buffer;
  static allocUnsafe(size: number): Buffer;
  static isBuffer(obj: any): obj is Buffer;
  static isEncoding(encoding: string): boolean;
  static byteLength(string: string, encoding?: BufferEncoding): number;
  static concat(
    list: (Buffer | Uint8Array)[],
    totalLength?: number,
  ): Buffer;
  static compare(buf1: Buffer, buf2: Buffer): number;

  toString(encoding?: BufferEncoding, start?: number, end?: number): string;
  toJSON(): { type: "Buffer"; data: number[] };
  equals(otherBuffer: Buffer | Uint8Array): boolean;
  compare(
    target: Buffer | Uint8Array,
    targetStart?: number,
    targetEnd?: number,
    sourceStart?: number,
    sourceEnd?: number,
  ): number;
  copy(
    target: Buffer | Uint8Array,
    targetStart?: number,
    sourceStart?: number,
    sourceEnd?: number,
  ): number;
  write(
    string: string,
    offset?: number,
    length?: number,
    encoding?: BufferEncoding,
  ): number;
  slice(start?: number, end?: number): Buffer;
  indexOf(
    value: number | string | Buffer | Uint8Array,
    byteOffset?: number,
    encoding?: BufferEncoding,
  ): number;
  includes(
    value: number | string | Buffer | Uint8Array,
    byteOffset?: number,
    encoding?: BufferEncoding,
  ): boolean;
  fill(
    value: number | string | Buffer | Uint8Array,
    offset?: number,
    end?: number,
    encoding?: BufferEncoding,
  ): this;

  readUInt8(offset?: number): number;
  readUInt16BE(offset?: number): number;
  readUInt16LE(offset?: number): number;
  readUInt32BE(offset?: number): number;
  readUInt32LE(offset?: number): number;
  readInt8(offset?: number): number;
  readInt16BE(offset?: number): number;
  readInt16LE(offset?: number): number;
  readInt32BE(offset?: number): number;
  readInt32LE(offset?: number): number;
  readFloatBE(offset?: number): number;
  readFloatLE(offset?: number): number;
  readDoubleBE(offset?: number): number;
  readDoubleLE(offset?: number): number;

  writeUInt8(value: number, offset?: number): number;
  writeUInt16BE(value: number, offset?: number): number;
  writeUInt16LE(value: number, offset?: number): number;
  writeUInt32BE(value: number, offset?: number): number;
  writeUInt32LE(value: number, offset?: number): number;
  writeInt8(value: number, offset?: number): number;
  writeInt16BE(value: number, offset?: number): number;
  writeInt16LE(value: number, offset?: number): number;
  writeInt32BE(value: number, offset?: number): number;
  writeInt32LE(value: number, offset?: number): number;
  writeFloatBE(value: number, offset?: number): number;
  writeFloatLE(value: number, offset?: number): number;
  writeDoubleBE(value: number, offset?: number): number;
  writeDoubleLE(value: number, offset?: number): number;
}

type BufferEncoding =
  | "utf8"
  | "utf-8"
  | "ascii"
  | "latin1"
  | "binary"
  | "base64"
  | "base64url"
  | "hex"
  | "ucs2"
  | "ucs-2"
  | "utf16le"
  | "utf-16le";
