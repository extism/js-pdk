/**
 * A simple example of a JavaScript, CJS flavored plug-in:
 */

function greet() {
  Host.outputString(`Hello, ${Host.inputString()}`)
}

module.exports = { greet }