// Extract host functions by name.
// Note: these must be declared in the d.ts file
const { myHostFunction1, myHostFunction2 } = Host.getFunctions()

function greet() {
  let msg = Memory.fromBuffer((new TextEncoder().encode("hello from js 1")).buffer)
  console.log(`MSG ${msg.offset} ${msg.len}`)
  let ptr = myHostFunction1(msg.offset)
  let response = new TextDecoder().decode(Memory.readBytes(ptr))
  console.log(`response: ${response}`)

  Host.outputString(`Hello, World!`)
}

module.exports = { greet }
