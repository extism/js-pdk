import { Host, Http, Var } from "../../../crates/core/src/prelude/src/index.ts";

export function greet() {
  Var.set("name", "MAYBESteve");
  let extra = new TextEncoder().encode("aaa")
  let decoded = new TextDecoder().decode(extra)
  const res = Http.request({ url: "https://example.com", method: "GET" });
  const name = Var.getString("name") || "unknown";
  Host.outputString(`Hello, ${Host.inputString()} (or is it ${name}???) ${decoded} ${new Date().toString()}\n\n${res.body}`)
}
