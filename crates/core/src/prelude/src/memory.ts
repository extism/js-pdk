declare global {
  interface Memory {
    fromString(str: string): MemoryHandle;
    fromBuffer(bytes: ArrayBufferLike): MemoryHandle;
    fromJsonObject(obj: JSON): MemoryHandle;
    allocUInt32(i: number): MemoryHandle;
    allocUInt64(i: bigint): MemoryHandle;
    allocFloat32(i: number): MemoryHandle;
    allocFloat64(i: number): MemoryHandle;
    find(offset: PTR): MemoryHandle;
    free(offset: PTR): void;

    _shared: ArrayBuffer;

    /**
     * @internal
     */
    _fromBuffer(buffer: ArrayBuffer): void;
    /**
     * @internal
     */
    _find(): void;
    /**
     * @internal
     */
    _free(): void;
    /**
     * @internal
     */
    _readBytes(offset: PTR): ArrayBuffer;
  }
  var Memory: Memory;
}

const Memory = globalThis.Memory;
const OFFSET_LEN = new DataView(Memory._shared);

Memory.fromString = function(this: Memory, str) {
  let bytes = new TextEncoder().encode(str).buffer;
  const memData = Memory.fromBuffer(bytes);
  return new MemoryHandle(memData.offset, memData.len);
};

Memory.fromBuffer = function(this: Memory, bytes) {
  Memory._fromBuffer(bytes);
  const offset = OFFSET_LEN.getBigUint64(0, true);
  const len = OFFSET_LEN.getBigUint64(8, true);
  return new MemoryHandle(offset, len);
};

Memory.fromJsonObject = function(this: Memory, obj) {
  const memData = Memory.fromString(JSON.stringify(obj));
  return new MemoryHandle(memData.offset, memData.len);
};

Memory.allocUInt32 = function(this: Memory, i) {
  const buffer = new ArrayBuffer(4);
  const arr = new Uint32Array(buffer);
  arr[0] = i;
  return Memory.fromBuffer(buffer);
};

Memory.allocUInt64 = function(this: Memory, i) {
  const buffer = new ArrayBuffer(8);
  const arr = new BigUint64Array(buffer);
  arr[0] = i;
  return Memory.fromBuffer(buffer);
};

Memory.allocFloat32 = function(this: Memory, i) {
  const buffer = new ArrayBuffer(4);
  const arr = new Float32Array(buffer);
  arr[0] = i;
  return Memory.fromBuffer(buffer);
};

Memory.allocFloat64 = function(this: Memory, i) {
  const buffer = new ArrayBuffer(8);
  const arr = new Float64Array(buffer);
  arr[0] = i;
  return Memory.fromBuffer(buffer);
};

Memory.find = function(offset) {
  OFFSET_LEN.setBigUint64(0, BigInt(offset), true);
  Memory._find();
  offset = OFFSET_LEN.getBigUint64(0, true);
  const len = OFFSET_LEN.getBigUint64(8, true);
  return new MemoryHandle(offset, len);
};

Memory.free = function(offset) {
  OFFSET_LEN.setBigUint64(0, BigInt(offset), true);
  Memory._free();
}

export { };
