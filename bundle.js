var __defProp = Object.defineProperty;
var __getOwnPropDesc = Object.getOwnPropertyDescriptor;
var __getOwnPropNames = Object.getOwnPropertyNames;
var __hasOwnProp = Object.prototype.hasOwnProperty;
var __export = (target, all) => {
  for (var name in all)
    __defProp(target, name, { get: all[name], enumerable: true });
};
var __copyProps = (to, from, except, desc) => {
  if (from && typeof from === "object" || typeof from === "function") {
    for (let key of __getOwnPropNames(from))
      if (!__hasOwnProp.call(to, key) && key !== except)
        __defProp(to, key, { get: () => from[key], enumerable: !(desc = __getOwnPropDesc(from, key)) || desc.enumerable });
  }
  return to;
};
var __toCommonJS = (mod) => __copyProps(__defProp({}, "__esModule", { value: true }), mod);

// script.js
var script_exports = {};
__export(script_exports, {
  call_http: () => call_http,
  count_vowels: () => count_vowels,
  greet: () => greet,
  greet2: () => greet2,
  i_error_out: () => i_error_out
});
module.exports = __toCommonJS(script_exports);
var VOWELS = [
  "a",
  "e",
  "i",
  "o",
  "u"
];
function privateFunction() {
  return 0;
}
function count_vowels() {
  let input = Host.inputString();
  let count = privateFunction();
  for (let i = 0; i < input.length; i++) {
    if (VOWELS.includes(input[i].toLowerCase())) {
      count += 1;
    }
  }
  Host.outputString(JSON.stringify({ count }));
  return 0;
}
function greet() {
  Host.outputString("Hello World from greet! " + Host.inputString());
  return 0;
}
function greet2() {
  console.log("console log");
  console.error("console error");
  Var.set("thing", "variable value");
  Host.outputBytes(Var.get("thing"));
  return 0;
}
function i_error_out() {
  throw Error("I am an error");
}
function call_http() {
  let body = JSON.stringify({
    "model": "gpt-3.5-turbo",
    "temperature": 0.7,
    "messages": [
      {
        "role": "user",
        "content": "Please write a haiku about Wasm"
      }
    ]
  });
  let resp = Http.request(
    {
      url: "https://api.openai.com/v1/chat/completions",
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "Authorization": `Bearer ${Config.get("open_ai_key")}`
      }
    },
    body
  );
  Host.outputString(resp.body);
}
