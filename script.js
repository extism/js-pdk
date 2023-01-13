const VOWELS = [
    'a', 'e', 'i', 'o', 'u',
]

function count_vowels() {
    let input = Host.inputString()
    let count = 0
    for (let i = 0; i < input.length; i++) {
        if (VOWELS.includes(input[i].toLowerCase())) {
            count += 1
        }
    }
    Host.outputString(JSON.stringify({count}))
    return 0
}

function greet() {
    Host.outputString("Hello World from greet! " + Host.inputString())
    return 0
}

function greet2() {
    Host.outputString("Hello World from greet2! " + Host.inputString())
    return 0
}

module.exports = { greet, count_vowels, greet2 }
