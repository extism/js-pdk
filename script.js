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
    Host.outputString(JSON.stringify({count}))
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

