declare global {
  interface Host {
    /**
     * @internal
     */
    __hostFunctions: Array<{ name: string; results: number }>;

    invokeFunc0(id: number, ...args: unknown[]): void;
    invokeFunc(id: number, ...args: unknown[]): unknown;
    inputBytes(): ArrayBufferLike;
    inputString(): string;
    outputBytes(output: ArrayBufferLike): boolean;
    outputString(output: string): boolean;
    getFunctions(): import("extism:host").user;
  }

  var Host: Host;
}

Host.getFunctions = function () {
  function createInvoke(id: number, results: number) {
    if (results === 0) {
      return Host.invokeFunc0.bind(null, id);
    } else {
      return Host.invokeFunc.bind(null, id);
    }
  }

  return Host.__hostFunctions.reduce((funcs, meta, id) => {
    funcs[meta.name] = createInvoke(id, meta.results);
    return funcs;
  }, {});
};

export {};
