/**
 * A simple example of a JavaScript, CJS flavored plug-in:
 */

function greet() {
  Host.outputString(`Hello, ${Host.inputString()}!`);
}

function goodbye() {
  Host.outputString(`Goodbye, ${Host.inputString()}!`);
}

module.exports = { greet, goodbye };
