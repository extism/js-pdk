declare global {
  /**
   * @internal
   */
  function __getTime(): string;
}

globalThis.Date = new Proxy(Date, {
  apply() {
    return __getTime();
  },
  construct(target, args) {
    if (args.length === 0) return new target(__getTime());

    return Reflect.construct(target, args);
  },
});

export {};
