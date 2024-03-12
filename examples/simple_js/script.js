/**
 * A simple example of a JavaScript, CJS flavored plug-in:
 */

function greet() {
  setTimeout(() => {
    Host.outputString(`and invited everyone you knew; you would see host string would be from me`)
  }, 2000);

  setTimeout(() => {
    Host.outputString(`and the card attached would read, "thank you for being ${Host.inputString()}"`)
  }, 2000);

  setTimeout(() => {
    Host.outputString(`and if you threw a party`)
  }, 100);
  Host.outputString(`thank you for being host string!`)
}

module.exports = { greet }
