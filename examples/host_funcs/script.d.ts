declare module 'main' {
  export function greet(): I32;
}

declare module 'extism:host' {
  interface user {
    capitalize(ptr: PTR): PTR;
    floatInputs(p1: F64, p2: F32): I32;
    floatOutput(p1: I32): F64;
    voidInputs(p1: I32, p2: I64, p3: F32, p4: F64, p5: I32): void;
  }
}
