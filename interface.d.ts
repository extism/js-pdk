declare module 'extism:host' {
  interface user {
    myHostFunction1(p: I32, q: I32): I32;
    myHostFunction2(p: I32): I32;
  }
}

declare module 'main' {
  // the exports
  export function greet(): I32;
}
