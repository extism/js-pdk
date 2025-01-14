/**
 * A simple example of a JavaScript, CJS flavored plug-in:
 */

function greet() {
  console.log('hello world!');
  console.log('hello', 'world!');
  console.log('hello', 'world', '!');
  console.log(1, 2, 3);
}


module.exports = { greet };
