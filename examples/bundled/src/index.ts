import { host } from "../../../crates/core/src/prelude/dist/index.js";

export function greet() {
  let extra = new TextEncoder().encode("aaa")
  let decoded = new TextDecoder().decode(extra)
  host.outputString(`Hello, ${host.inputString()} ${decoded} ${new Date().toString()}`)
}
