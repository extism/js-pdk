declare module 'main' {
  export function add3(a: I32, b: I32, c: I32): I32;
  export function appendString(a: I64, b: I64): string;
}

declare module 'extism:host' {
  interface user {
    testing123(a: string): {a: string};
  }
}
