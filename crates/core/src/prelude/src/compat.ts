globalThis.self = globalThis;

globalThis.queueMicrotask = function queueMicrotask(callback: () => void): void {
  callback();
};

export {};
