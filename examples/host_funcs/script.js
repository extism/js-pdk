// Extract host functions by name.
// Note: these must be declared in the d.ts file
const { myHostFunction1, myHostFunction2 } = Host.getFunctions()

function greet() {
  let msg = "Hello from js 1"
  let mem = Memory.fromBuffer((new TextEncoder().encode(msg)).buffer)
  let ptr = myHostFunction1(mem.offset)
  let response = new TextDecoder().decode(Memory.readBytes(ptr))
  if (response != "myHostFunction1: " + msg) {
    throw Error(`wrong message came back from myHostFunction1: ${response}`)
  }

  msg = "Hello from js 2"
  mem = Memory.fromBuffer((new TextEncoder().encode(msg)).buffer)
  ptr = myHostFunction2(mem.offset)
  response = new TextDecoder().decode(Memory.readBytes(ptr))
  if (response != "myHostFunction2: " + msg) {
    throw Error(`wrong message came back from myHostFunction2: ${response}`)
  }

  Host.outputString(`Hello, World!`)
}

module.exports = { greet }
