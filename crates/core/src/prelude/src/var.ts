declare global {
  var Var: {
    set(name: string, value: string | ArrayBufferLike): void;
    getBytes(name: string): ArrayBufferLike | null;
    getString(name: string): string | null;
  };
}

export {};
