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

    /**
     * @internal
     */
    _fromBuffer(buffer: ArrayBuffer): { offset: PTR; len: I64 };
    /**
     * @internal
     */
    _find(offset: PTR): { offset: PTR; len: I64 };
    /**
     * @internal
     */
    _free(offset: PTR): void;
    /**
     * @internal
     */
    _readBytes(offset: PTR): ArrayBuffer;
  }
  var Memory: Memory;
}

const Memory = globalThis.Memory;
Memory.fromString = function(this: Memory, str) {
  // todo validate
  let bytes = new TextEncoder().encode(str).buffer;
  const memData = Memory.fromBuffer(bytes);
  return new MemoryHandle(memData.offset, memData.len);
};

Memory.fromBuffer = function(this: Memory, bytes) {
  // todo validate
  const memData = Memory._fromBuffer(bytes);
  return new MemoryHandle(memData.offset, memData.len);
};

Memory.fromJsonObject = function(this: Memory, obj) {
  // todo validate
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
  // todo validate
  const memData = Memory._find(offset);
  return new MemoryHandle(memData.offset, memData.len);
};

export { };
