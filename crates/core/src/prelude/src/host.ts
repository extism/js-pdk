declare global {
  interface Host {
    /**
     * @internal
     */
    __hostFunctions: Array<{ name: string; results: number }>;

    invokeFunc(id: number, ...args: unknown[]): number;
    inputBytes(): ArrayBufferLike;
    inputString(): string;
    outputBytes(output: ArrayBufferLike): boolean;
    outputString(output: string): boolean;
    getFunctions(): import("extism:host").user;
    arrayBufferToBase64(input: ArrayBuffer): string;
    base64ToArrayBuffer(input: string): ArrayBuffer;
  }

  var Host: Host;
}

Host.getFunctions = function () {
  return Host.__hostFunctions.reduce((funcs, meta, id) => {
    funcs[meta.name] = (...args: unknown[]) => {
      const sanitizedArgs = args.map(arg => 
        arg === undefined || arg === null ? 0 : arg
      );
      const result = Host.invokeFunc(id, ...sanitizedArgs); 
      return meta.results === 0 ? undefined : result;
    };
    return funcs;
  }, {});
};

export { };