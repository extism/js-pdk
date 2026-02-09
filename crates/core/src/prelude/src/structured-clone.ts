declare global {
  function structuredClone<T>(value: T): T;
}

function cloneValue(value: any, seen: Map<any, any>): any {
  if (value === null || typeof value !== "object") {
    return value;
  }

  if (seen.has(value)) {
    return seen.get(value);
  }

  if (value instanceof Date) {
    return new Date(value.getTime());
  }

  if (value instanceof RegExp) {
    return new RegExp(value.source, value.flags);
  }

  if (value instanceof ArrayBuffer) {
    return value.slice(0);
  }

  if (ArrayBuffer.isView(value)) {
    const clonedBuffer = (value as any).buffer.slice(0);
    const Ctor = value.constructor as any;
    return new Ctor(
      clonedBuffer,
      value.byteOffset,
      (value as any).length !== undefined
        ? (value as any).length
        : value.byteLength
    );
  }

  if (value instanceof Map) {
    const result = new Map();
    seen.set(value, result);
    Array.from(value).forEach(function (entry: any) {
      result.set(cloneValue(entry[0], seen), cloneValue(entry[1], seen));
    });
    return result;
  }

  if (value instanceof Set) {
    const result = new Set();
    seen.set(value, result);
    Array.from(value).forEach(function (v: any) {
      result.add(cloneValue(v, seen));
    });
    return result;
  }

  if (Array.isArray(value)) {
    const result: any[] = [];
    seen.set(value, result);
    for (let i = 0; i < value.length; i++) {
      result[i] = cloneValue(value[i], seen);
    }
    return result;
  }

  if (value instanceof Error) {
    const result = new (value.constructor as any)(value.message);
    seen.set(value, result);
    result.stack = value.stack;
    return result;
  }

  // Plain objects
  const result: Record<string, any> = {};
  seen.set(value, result);
  for (const key of Object.keys(value)) {
    result[key] = cloneValue(value[key], seen);
  }
  return result;
}

globalThis.structuredClone = function structuredClone<T>(value: T): T {
  return cloneValue(value, new Map());
};

export {};
