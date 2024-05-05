/**
 * A simple example of generate non-plugin function exports
 */

function add3(a, b, c) {
  return a + b + c;
}

function appendString(a, b) {
  a = Memory.find(a).readString();
  b = Memory.find(b).readString();
  return Memory.fromString(a + b).offset;
}

module.exports = { add3, appendString };
