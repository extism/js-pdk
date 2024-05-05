declare global {
  interface MemoryHandle {
    offset: number;
    len: number;

    readString(): string;
    readUInt32(): number;
    readUInt64(): bigint;
    readFloat32(): number;
    readUFloat64(): number;
    readBytes(): ArrayBuffer;
    readJsonObject<T = any>(): T;
    free(): void;
  }

  var MemoryHandle: {
    prototype: MemoryHandle;
    new (offset: number, len: number): MemoryHandle;
  };
}

class MemoryHandle implements globalThis.MemoryHandle {
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
    return Memory._readBytes(this.offset);
  }

  readJsonObject() {
    return JSON.parse(this.readString());
  }

  free() {
    Memory._free(this.offset);
  }
}

globalThis.MemoryHandle = MemoryHandle;

export {};
