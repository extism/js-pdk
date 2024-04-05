declare module 'main' {
  export function greet(): I32;
}

declare module 'extism:host' {
  interface user {
    myHostFunction1(ptr: PTR): PTR;
    myHostFunction2(ptr: PTR): PTR;
  }
}
