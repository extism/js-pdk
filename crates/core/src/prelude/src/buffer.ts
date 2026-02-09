// Buffer polyfill for Extism JS PDK
// Implements Node.js Buffer API on top of Uint8Array

function normalizeEncoding(enc?: string): string {
  if (!enc) return "utf8";
  const lower = enc.toLowerCase();
  switch (lower) {
    case "utf8":
    case "utf-8":
      return "utf8";
    case "ascii":
      return "ascii";
    case "latin1":
    case "binary":
      return "latin1";
    case "base64":
      return "base64";
    case "base64url":
      return "base64url";
    case "hex":
      return "hex";
    case "ucs2":
    case "ucs-2":
    case "utf16le":
    case "utf-16le":
      return "utf16le";
    default:
      throw new TypeError("Unknown encoding: " + enc);
  }
}

function encodingToBytes(str: string, encoding: string): Uint8Array {
  switch (encoding) {
    case "utf8": {
      return new TextEncoder().encode(str);
    }
    case "ascii":
    case "latin1": {
      const bytes = new Uint8Array(str.length);
      for (let i = 0; i < str.length; i++) {
        bytes[i] = str.charCodeAt(i) & 0xff;
      }
      return bytes;
    }
    case "base64": {
      // Handle base64 with whitespace and missing padding
      let cleaned = str.replace(/[\s\r\n]+/g, "");
      while (cleaned.length % 4 !== 0) cleaned += "=";
      const binary = atob(cleaned);
      const bytes = new Uint8Array(binary.length);
      for (let i = 0; i < binary.length; i++) {
        bytes[i] = binary.charCodeAt(i);
      }
      return bytes;
    }
    case "base64url": {
      let b64 = str.replace(/-/g, "+").replace(/_/g, "/");
      while (b64.length % 4 !== 0) b64 += "=";
      return encodingToBytes(b64, "base64");
    }
    case "hex": {
      if (str.length % 2 !== 0) throw new TypeError("Invalid hex string");
      const bytes = new Uint8Array(str.length / 2);
      for (let i = 0; i < str.length; i += 2) {
        bytes[i / 2] = parseInt(str.substring(i, i + 2), 16);
      }
      return bytes;
    }
    case "utf16le": {
      const bytes = new Uint8Array(str.length * 2);
      for (let i = 0; i < str.length; i++) {
        const code = str.charCodeAt(i);
        bytes[i * 2] = code & 0xff;
        bytes[i * 2 + 1] = (code >> 8) & 0xff;
      }
      return bytes;
    }
    default:
      throw new TypeError("Unknown encoding: " + encoding);
  }
}

function bytesToEncoding(
  bytes: Uint8Array,
  encoding: string,
  start: number,
  end: number,
): string {
  const slice = bytes.subarray(start, end);
  switch (encoding) {
    case "utf8": {
      return new TextDecoder().decode(slice);
    }
    case "ascii": {
      let str = "";
      for (let i = 0; i < slice.length; i++) {
        str += String.fromCharCode(slice[i] & 0x7f);
      }
      return str;
    }
    case "latin1": {
      let str = "";
      for (let i = 0; i < slice.length; i++) {
        str += String.fromCharCode(slice[i]);
      }
      return str;
    }
    case "base64": {
      let binary = "";
      for (let i = 0; i < slice.length; i++) {
        binary += String.fromCharCode(slice[i]);
      }
      return btoa(binary);
    }
    case "base64url": {
      return bytesToEncoding(bytes, "base64", start, end)
        .replace(/\+/g, "-")
        .replace(/\//g, "_")
        .replace(/=+$/, "");
    }
    case "hex": {
      let hex = "";
      for (let i = 0; i < slice.length; i++) {
        hex += (slice[i] < 16 ? "0" : "") + slice[i].toString(16);
      }
      return hex;
    }
    case "utf16le": {
      let str = "";
      for (let i = 0; i + 1 < slice.length; i += 2) {
        str += String.fromCharCode(slice[i] | (slice[i + 1] << 8));
      }
      return str;
    }
    default:
      throw new TypeError("Unknown encoding: " + encoding);
  }
}

class _Buffer extends Uint8Array {
  static get [Symbol.species](): typeof Uint8Array {
    return Uint8Array;
  }

  static from(
    value: any,
    encodingOrOffset?: any,
    length?: number,
  ): _Buffer {
    if (typeof value === "string") {
      const encoding = normalizeEncoding(encodingOrOffset as string);
      const bytes = encodingToBytes(value, encoding);
      const buf = new _Buffer(bytes.length);
      buf.set(bytes);
      return buf;
    }

    if (value instanceof ArrayBuffer) {
      const offset = (encodingOrOffset as number) || 0;
      const len =
        length !== undefined ? length : value.byteLength - offset;
      return new _Buffer(value, offset, len);
    }

    if (_Buffer.isBuffer(value)) {
      const buf = new _Buffer(value.length);
      buf.set(value);
      return buf;
    }

    if (ArrayBuffer.isView(value)) {
      const src = new Uint8Array(
        value.buffer,
        value.byteOffset,
        value.byteLength,
      );
      const buf = new _Buffer(src.length);
      buf.set(src);
      return buf;
    }

    if (
      typeof value === "object" &&
      value !== null &&
      value.type === "Buffer" &&
      Array.isArray(value.data)
    ) {
      return _Buffer.from(value.data);
    }

    if (
      Array.isArray(value) ||
      (typeof value === "object" &&
        value !== null &&
        typeof value.length === "number")
    ) {
      const buf = new _Buffer(value.length);
      for (let i = 0; i < value.length; i++) {
        buf[i] = value[i] & 0xff;
      }
      return buf;
    }

    throw new TypeError(
      "The first argument must be a string, Buffer, ArrayBuffer, Array, or array-like object",
    );
  }

  static alloc(size: number, fill?: any, encoding?: string): _Buffer {
    const buf = new _Buffer(size);
    if (fill !== undefined) {
      buf.fill(fill, 0, size, encoding);
    }
    return buf;
  }

  static allocUnsafe(size: number): _Buffer {
    return new _Buffer(size);
  }

  static isBuffer(obj: any): obj is _Buffer {
    return obj instanceof _Buffer;
  }

  static isEncoding(encoding: string): boolean {
    try {
      normalizeEncoding(encoding);
      return true;
    } catch {
      return false;
    }
  }

  static byteLength(string: string, encoding?: string): number {
    const enc = normalizeEncoding(encoding);
    return encodingToBytes(string, enc).length;
  }

  static concat(
    list: (Uint8Array | _Buffer)[],
    totalLength?: number,
  ): _Buffer {
    if (totalLength === undefined) {
      totalLength = 0;
      for (let i = 0; i < list.length; i++) {
        totalLength += list[i].length;
      }
    }
    const buf = _Buffer.alloc(totalLength);
    let offset = 0;
    for (let i = 0; i < list.length; i++) {
      const item = list[i];
      if (offset + item.length > totalLength!) {
        buf.set(item.subarray(0, totalLength! - offset), offset);
        break;
      }
      buf.set(item, offset);
      offset += item.length;
    }
    return buf;
  }

  static compare(buf1: _Buffer, buf2: _Buffer): number {
    const len = Math.min(buf1.length, buf2.length);
    for (let i = 0; i < len; i++) {
      if (buf1[i] < buf2[i]) return -1;
      if (buf1[i] > buf2[i]) return 1;
    }
    if (buf1.length < buf2.length) return -1;
    if (buf1.length > buf2.length) return 1;
    return 0;
  }

  // @ts-ignore - wider signature than Uint8Array.toString
  toString(encoding?: string, start?: number, end?: number): string {
    const enc = normalizeEncoding(encoding);
    const s = start || 0;
    const e = end !== undefined ? end : this.length;
    return bytesToEncoding(this, enc, s, e);
  }

  toJSON(): { type: string; data: number[] } {
    return { type: "Buffer", data: Array.from(this) };
  }

  equals(otherBuffer: _Buffer | Uint8Array): boolean {
    if (this.length !== otherBuffer.length) return false;
    for (let i = 0; i < this.length; i++) {
      if (this[i] !== otherBuffer[i]) return false;
    }
    return true;
  }

  compare(
    target: _Buffer | Uint8Array,
    targetStart?: number,
    targetEnd?: number,
    sourceStart?: number,
    sourceEnd?: number,
  ): number {
    const tStart = targetStart || 0;
    const tEnd = targetEnd !== undefined ? targetEnd : target.length;
    const sStart = sourceStart || 0;
    const sEnd = sourceEnd !== undefined ? sourceEnd : this.length;

    const src = this.subarray(sStart, sEnd);
    const tgt = target.subarray(tStart, tEnd);

    const len = Math.min(src.length, tgt.length);
    for (let i = 0; i < len; i++) {
      if (src[i] < tgt[i]) return -1;
      if (src[i] > tgt[i]) return 1;
    }
    if (src.length < tgt.length) return -1;
    if (src.length > tgt.length) return 1;
    return 0;
  }

  copy(
    target: _Buffer | Uint8Array,
    targetStart?: number,
    sourceStart?: number,
    sourceEnd?: number,
  ): number {
    const tStart = targetStart || 0;
    const sStart = sourceStart || 0;
    const sEnd = sourceEnd !== undefined ? sourceEnd : this.length;
    const src = this.subarray(sStart, sEnd);
    const toCopy = Math.min(src.length, target.length - tStart);
    target.set(src.subarray(0, toCopy), tStart);
    return toCopy;
  }

  write(
    string: string,
    offsetOrEncoding?: number | string,
    lengthOrEncoding?: number | string,
    encoding?: string,
  ): number {
    let off = 0;
    let enc = "utf8";
    let len: number | undefined;

    if (typeof offsetOrEncoding === "string") {
      enc = normalizeEncoding(offsetOrEncoding);
    } else if (typeof offsetOrEncoding === "number") {
      off = offsetOrEncoding;
      if (typeof lengthOrEncoding === "string") {
        enc = normalizeEncoding(lengthOrEncoding);
      } else if (typeof lengthOrEncoding === "number") {
        len = lengthOrEncoding;
        if (encoding) {
          enc = normalizeEncoding(encoding);
        }
      }
    }

    const bytes = encodingToBytes(string, enc);
    const maxLen = this.length - off;
    const toCopy = Math.min(
      len !== undefined ? Math.min(len, bytes.length) : bytes.length,
      maxLen,
    );
    this.set(bytes.subarray(0, toCopy), off);
    return toCopy;
  }

  // Node.js Buffer.slice returns a view (shared memory), not a copy
  // @ts-ignore - return type differs from Uint8Array.slice
  slice(start?: number, end?: number): _Buffer {
    const sub = super.subarray(start, end);
    Object.setPrototypeOf(sub, _Buffer.prototype);
    return sub as unknown as _Buffer;
  }

  // @ts-ignore - wider signature than Uint8Array.indexOf
  indexOf(
    value: any,
    byteOffset?: number,
    encoding?: string,
  ): number {
    const offset = byteOffset || 0;
    if (typeof value === "number") {
      for (let i = offset; i < this.length; i++) {
        if (this[i] === (value & 0xff)) return i;
      }
      return -1;
    }
    let needle: Uint8Array;
    if (typeof value === "string") {
      const enc = normalizeEncoding(encoding);
      needle = encodingToBytes(value, enc);
    } else {
      needle = value as Uint8Array;
    }
    if (needle.length === 0) return offset;
    for (let i = offset; i <= this.length - needle.length; i++) {
      let found = true;
      for (let j = 0; j < needle.length; j++) {
        if (this[i + j] !== needle[j]) {
          found = false;
          break;
        }
      }
      if (found) return i;
    }
    return -1;
  }

  // @ts-ignore - wider signature than Uint8Array.includes
  includes(
    value: any,
    byteOffset?: number,
    encoding?: string,
  ): boolean {
    return this.indexOf(value, byteOffset, encoding) !== -1;
  }

  // @ts-ignore - wider signature than Uint8Array.fill
  fill(
    value: any,
    offset?: number,
    end?: number,
    encoding?: string,
  ): this {
    const off = offset || 0;
    const e = end !== undefined ? end : this.length;
    if (typeof value === "number") {
      super.fill(value & 0xff, off, e);
      return this;
    }
    if (typeof value === "string") {
      if (value.length === 0) {
        super.fill(0, off, e);
        return this;
      }
      const enc = normalizeEncoding(encoding);
      const bytes = encodingToBytes(value, enc);
      if (bytes.length === 1) {
        super.fill(bytes[0], off, e);
        return this;
      }
      for (let i = off; i < e; i++) {
        this[i] = bytes[(i - off) % bytes.length];
      }
      return this;
    }
    if (value instanceof Uint8Array) {
      for (let i = off; i < e; i++) {
        this[i] = value[(i - off) % value.length];
      }
      return this;
    }
    throw new TypeError(
      "value must be a number, string, Buffer, or Uint8Array",
    );
  }

  // Read methods
  readUInt8(offset: number = 0): number {
    return this[offset];
  }
  readUInt16BE(offset: number = 0): number {
    return (this[offset] << 8) | this[offset + 1];
  }
  readUInt16LE(offset: number = 0): number {
    return this[offset] | (this[offset + 1] << 8);
  }
  readUInt32BE(offset: number = 0): number {
    return (
      (this[offset] * 0x1000000 +
        ((this[offset + 1] << 16) |
          (this[offset + 2] << 8) |
          this[offset + 3])) >>>
      0
    );
  }
  readUInt32LE(offset: number = 0): number {
    return (
      (this[offset] |
        (this[offset + 1] << 8) |
        (this[offset + 2] << 16) |
        (this[offset + 3] * 0x1000000)) >>>
      0
    );
  }
  readInt8(offset: number = 0): number {
    const val = this[offset];
    return val & 0x80 ? val - 0x100 : val;
  }
  readInt16BE(offset: number = 0): number {
    const val = (this[offset] << 8) | this[offset + 1];
    return val & 0x8000 ? val - 0x10000 : val;
  }
  readInt16LE(offset: number = 0): number {
    const val = this[offset] | (this[offset + 1] << 8);
    return val & 0x8000 ? val - 0x10000 : val;
  }
  readInt32BE(offset: number = 0): number {
    return (
      (this[offset] << 24) |
      (this[offset + 1] << 16) |
      (this[offset + 2] << 8) |
      this[offset + 3]
    );
  }
  readInt32LE(offset: number = 0): number {
    return (
      this[offset] |
      (this[offset + 1] << 8) |
      (this[offset + 2] << 16) |
      (this[offset + 3] << 24)
    );
  }
  readFloatBE(offset: number = 0): number {
    const dv = new DataView(this.buffer, this.byteOffset, this.byteLength);
    return dv.getFloat32(offset, false);
  }
  readFloatLE(offset: number = 0): number {
    const dv = new DataView(this.buffer, this.byteOffset, this.byteLength);
    return dv.getFloat32(offset, true);
  }
  readDoubleBE(offset: number = 0): number {
    const dv = new DataView(this.buffer, this.byteOffset, this.byteLength);
    return dv.getFloat64(offset, false);
  }
  readDoubleLE(offset: number = 0): number {
    const dv = new DataView(this.buffer, this.byteOffset, this.byteLength);
    return dv.getFloat64(offset, true);
  }

  // Write methods
  writeUInt8(value: number, offset: number = 0): number {
    this[offset] = value & 0xff;
    return offset + 1;
  }
  writeUInt16BE(value: number, offset: number = 0): number {
    this[offset] = (value >>> 8) & 0xff;
    this[offset + 1] = value & 0xff;
    return offset + 2;
  }
  writeUInt16LE(value: number, offset: number = 0): number {
    this[offset] = value & 0xff;
    this[offset + 1] = (value >>> 8) & 0xff;
    return offset + 2;
  }
  writeUInt32BE(value: number, offset: number = 0): number {
    this[offset] = (value >>> 24) & 0xff;
    this[offset + 1] = (value >>> 16) & 0xff;
    this[offset + 2] = (value >>> 8) & 0xff;
    this[offset + 3] = value & 0xff;
    return offset + 4;
  }
  writeUInt32LE(value: number, offset: number = 0): number {
    this[offset] = value & 0xff;
    this[offset + 1] = (value >>> 8) & 0xff;
    this[offset + 2] = (value >>> 16) & 0xff;
    this[offset + 3] = (value >>> 24) & 0xff;
    return offset + 4;
  }
  writeInt8(value: number, offset: number = 0): number {
    this[offset] = value & 0xff;
    return offset + 1;
  }
  writeInt16BE(value: number, offset: number = 0): number {
    this[offset] = (value >>> 8) & 0xff;
    this[offset + 1] = value & 0xff;
    return offset + 2;
  }
  writeInt16LE(value: number, offset: number = 0): number {
    this[offset] = value & 0xff;
    this[offset + 1] = (value >>> 8) & 0xff;
    return offset + 2;
  }
  writeInt32BE(value: number, offset: number = 0): number {
    this[offset] = (value >>> 24) & 0xff;
    this[offset + 1] = (value >>> 16) & 0xff;
    this[offset + 2] = (value >>> 8) & 0xff;
    this[offset + 3] = value & 0xff;
    return offset + 4;
  }
  writeInt32LE(value: number, offset: number = 0): number {
    this[offset] = value & 0xff;
    this[offset + 1] = (value >>> 8) & 0xff;
    this[offset + 2] = (value >>> 16) & 0xff;
    this[offset + 3] = (value >>> 24) & 0xff;
    return offset + 4;
  }
  writeFloatBE(value: number, offset: number = 0): number {
    const dv = new DataView(this.buffer, this.byteOffset, this.byteLength);
    dv.setFloat32(offset, value, false);
    return offset + 4;
  }
  writeFloatLE(value: number, offset: number = 0): number {
    const dv = new DataView(this.buffer, this.byteOffset, this.byteLength);
    dv.setFloat32(offset, value, true);
    return offset + 4;
  }
  writeDoubleBE(value: number, offset: number = 0): number {
    const dv = new DataView(this.buffer, this.byteOffset, this.byteLength);
    dv.setFloat64(offset, value, false);
    return offset + 8;
  }
  writeDoubleLE(value: number, offset: number = 0): number {
    const dv = new DataView(this.buffer, this.byteOffset, this.byteLength);
    dv.setFloat64(offset, value, true);
    return offset + 8;
  }
}

(globalThis as any).Buffer = _Buffer;
