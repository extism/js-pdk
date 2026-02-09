declare global {
  /**
   * @internal
   */
  function __encodeStringToUtf8Buffer(input: string): ArrayBufferLike;
}

class TextEncoder implements globalThis.TextEncoder {
  readonly encoding: string;

  constructor() {
    this.encoding = "utf-8";
  }

  encode(input: string = ""): Uint8Array {
    input = input.toString(); // non-string inputs are converted to strings
    return new Uint8Array(__encodeStringToUtf8Buffer(input));
  }

  encodeInto(
    source: string,
    destination: Uint8Array
  ): TextEncoderEncodeIntoResult {
    const encoded = this.encode(source);
    const written = Math.min(encoded.length, destination.length);
    destination.set(encoded.subarray(0, written));
    // Count how many complete UTF-8 characters we wrote.
    // Walk back if we split a multi-byte sequence.
    let bytesWritten = written;
    if (bytesWritten < encoded.length && bytesWritten > 0) {
      // If the last byte written starts a multi-byte sequence that wasn't fully written,
      // we need to walk back to find the last complete character boundary.
      while (bytesWritten > 0 && (encoded[bytesWritten] & 0xc0) === 0x80) {
        bytesWritten--;
      }
      if (bytesWritten > 0 && bytesWritten < written) {
        // We split a character, remove the incomplete one
        const leadByte = encoded[bytesWritten - 1];
        let expectedLen = 1;
        if ((leadByte & 0xe0) === 0xc0) expectedLen = 2;
        else if ((leadByte & 0xf0) === 0xe0) expectedLen = 3;
        else if ((leadByte & 0xf8) === 0xf0) expectedLen = 4;
        const available = written - (bytesWritten - 1);
        if (available < expectedLen) {
          bytesWritten--;
        } else {
          bytesWritten = written;
        }
      }
    }
    // Count code units read by decoding the written bytes back
    let read = 0;
    if (bytesWritten > 0) {
      const writtenSlice = encoded.subarray(0, bytesWritten);
      const decoded = new TextDecoder().decode(writtenSlice);
      read = decoded.length;
    }
    // Zero out any trailing bytes we walked back from
    for (let i = bytesWritten; i < written; i++) {
      destination[i] = 0;
    }
    return { read, written: bytesWritten };
  }
}

globalThis.TextEncoder = TextEncoder;

export {};
