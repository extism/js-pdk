import React from 'react';

const VOWELS = [
  'a', 'e', 'i', 'o', 'u',
]

function privateFunction() {
  return 0
}

export function count_vowels() {
  let input = Host.inputString()
  let count = privateFunction()
  for (let i = 0; i < input.length; i++) {
    if (VOWELS.includes(input[i].toLowerCase())) {
      count += 1
    }
  }
  Host.outputString(JSON.stringify({ count }))
  return 0
}

export function greet() {
  Host.outputString("Hello World from greet! " + Host.inputString())
  return 0
}

export function greet2() {
  console.log("console log")
  console.error("console error")
  Var.set("thing", "variable value")
  Host.outputBytes(Var.get("thing"))
  return 0
}

export function i_error_out() {
  throw Error("I am an error")
}

export function call_http() {
  let body = JSON.stringify({
    "model": "gpt-3.5-turbo",
    "temperature": 0.7,
    "messages": [
      {
        "role": "user",
        "content": "Please write a haiku about Wasm",
      }
    ],
  })
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
  )
  Host.outputString(resp.body)
}

export function evalcode() {
  console.log(React.toString())
  Host.outputString(eval(Host.inputString()).toString())
}


