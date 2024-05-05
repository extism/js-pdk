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
    _source: string,
    _destination: Uint8Array
  ): TextEncoderEncodeIntoResult {
    throw new Error("encodeInto is not supported");
  }
}

globalThis.TextEncoder = TextEncoder;

export {};
