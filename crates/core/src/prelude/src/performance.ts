declare global {
  /**
   * @internal
   */
  function __getTimeMs(): number;

  interface Performance {
    now(): number;
    readonly timeOrigin: number;
  }

  var performance: Performance;
}

const timeOrigin = __getTimeMs();

globalThis.performance = {
  timeOrigin,
  now(): number {
    return __getTimeMs() - timeOrigin;
  },
};

export {};
