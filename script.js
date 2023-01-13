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
    return JSON.stringify({count})
}

function greet() {
    return "Hello World from greet! " + Host.inputString()
}

function greet2() {
    return "Hello World from greet2! " + Host.inputString()
}

module.exports = { greet, count_vowels, greet2 }
