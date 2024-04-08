import {
  Config,
  Host,
  Http,
  Var,
} from "../../../crates/core/src/prelude/src/index.ts";
export function greet() {
  Var.set("name", "MAYBESteve");
  let extra = new TextEncoder().encode("aaa");
  let decoded = new TextDecoder().decode(extra);
  const res = Http.request({ url: "https://example.com", method: "GET" });
  const name = Var.getString("name") || "unknown";
  const apiKey = Config.get("SOME_API_KEY") || "unknown";

  Host.outputString(
    `Hello, ${Host.inputString()} (or is it ${name}???) ${decoded} ${
      new Date().toString()
    }\n\n${res.body}\n\n ==== KEY: ${apiKey}`,
  );
}

// test this bundled.wasm like so:
// extism call ../bundled.wasm greet --input "steve" --wasi --allow-host "*" --config SOME_API_KEY=123456789
