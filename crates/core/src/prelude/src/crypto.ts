declare var __getRandomBytes: (n: number) => ArrayBuffer;
declare var __shaDigest: (algorithm: string, data: ArrayBuffer) => ArrayBuffer;

const _crypto = {
  getRandomValues<T extends ArrayBufferView>(array: T): T {
    if (
      !(array instanceof Int8Array) &&
      !(array instanceof Uint8Array) &&
      !(array instanceof Uint8ClampedArray) &&
      !(array instanceof Int16Array) &&
      !(array instanceof Uint16Array) &&
      !(array instanceof Int32Array) &&
      !(array instanceof Uint32Array) &&
      !(array instanceof BigInt64Array) &&
      !(array instanceof BigUint64Array)
    ) {
      throw new Error(
        "TypeMismatchError: The provided ArrayBufferView is not an integer type",
      );
    }

    if (array.byteLength > 65536) {
      throw new Error(
        "QuotaExceededError: The ArrayBufferView's byte length exceeds 65536",
      );
    }

    const randomBuffer = __getRandomBytes(array.byteLength);
    const randomBytes = new Uint8Array(randomBuffer);
    const target = new Uint8Array(array.buffer, array.byteOffset, array.byteLength);

    for (let i = 0; i < randomBytes.length; i++) {
      target[i] = randomBytes[i];
    }

    return array;
  },

  subtle: {
    digest(
      algorithm: string | { name: string },
      data: ArrayBuffer | ArrayBufferView,
    ): Promise<ArrayBuffer> {
      const algoName =
        typeof algorithm === "string" ? algorithm : algorithm.name;

      let buffer: ArrayBuffer;
      if (ArrayBuffer.isView(data)) {
        buffer = data.buffer.slice(
          data.byteOffset,
          data.byteOffset + data.byteLength,
        );
      } else {
        buffer = data;
      }

      const result = __shaDigest(algoName, buffer);
      return Promise.resolve(result);
    },
  },

  randomUUID(): string {
    const bytes = new Uint8Array(__getRandomBytes(16));

    // Set version 4 (0100xxxx in byte 6)
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    // Set variant 1 (10xxxxxx in byte 8)
    bytes[8] = (bytes[8] & 0x3f) | 0x80;

    const hex = Array.prototype.map
      .call(bytes, (b: number) => b.toString(16).padStart(2, "0"))
      .join("");

    return (
      hex.slice(0, 8) +
      "-" +
      hex.slice(8, 12) +
      "-" +
      hex.slice(12, 16) +
      "-" +
      hex.slice(16, 20) +
      "-" +
      hex.slice(20, 32)
    );
  },
};

globalThis.crypto = _crypto as any;

export {};
