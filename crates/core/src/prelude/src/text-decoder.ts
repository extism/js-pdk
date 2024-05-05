declare global {
  /**
   * @internal
   */
  function __decodeUtf8BufferToString(
    input: ArrayBufferLike,
    byteOffset: number,
    byteLength: number,
    fatal: boolean,
    ignoreBOM: boolean
  ): string;
}

class TextDecoder implements globalThis.TextDecoder {
  readonly encoding: string;
  readonly fatal: boolean;
  readonly ignoreBOM: boolean;

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

    this.encoding = "utf-8";
    this.fatal = !!options.fatal;
    this.ignoreBOM = !!options.ignoreBOM;
  }

  decode(
    input?: AllowSharedBufferSource,
    options: TextDecodeOptions = {}
  ): string {
    if (input === undefined) {
      return "";
    }

    if (options.stream) {
      throw new Error("Streaming decode is not supported");
    }

    // backing buffer would not have byteOffset and may have different byteLength
    let byteOffset = 0;
    let byteLength = input.byteLength;
    if (ArrayBuffer.isView(input)) {
      byteOffset = input.byteOffset;
      input = input.buffer;
    }

    if (!(input instanceof ArrayBuffer)) {
      throw new TypeError(
        "The provided value is not of type '(ArrayBuffer or ArrayBufferView)'"
      );
    }

    return __decodeUtf8BufferToString(
      input,
      byteOffset,
      byteLength,
      this.fatal,
      this.ignoreBOM
    );
  }
}

globalThis.TextDecoder = TextDecoder;

export {};
