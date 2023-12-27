declare module 'main' {
  export function greet(): I32;
}

declare module 'extism:host' {
  interface user {
    myHostFunction1(p: I32): I32;
    myHostFunction2(p: I32): I32;
  }
}
