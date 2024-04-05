import { Host, Http } from "../../../crates/core/src/prelude/src/index.ts";

export function greet() {
  let extra = new TextEncoder().encode("aaa")
  let decoded = new TextDecoder().decode(extra)
  const res = Http.request({ url: "https://example.com", method: "GET" });
  Host.outputString(`Hello, ${Host.inputString()} ${decoded} ${new Date().toString()}\n\n${res.body}`)
}
