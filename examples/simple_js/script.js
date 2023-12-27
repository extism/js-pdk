/**
 * A simple example of a JavaScript, CJS flavored plug-in:
 */

const { myHostFunc1, myHostFunc2 } = Host.getFunctions()

function greet() {
  let msg = Memory.fromBuffer((new TextEncoder().encode("hello from js 1")).buffer)
  console.log(`MSG ${msg.offset} ${msg.len}`)
  let ret = myHostFunc1(msg.offset)
  console.log(`Ret: ${ret}`)
  msg = Memory.find(ret)
  console.log(`Ret: ${msg}`)

  // let msg2 = Memory.fromBuffer((new TextEncoder().encode("hello from js 2")).buffer)
  // console.log(`MSG ${msg2.offset} ${msg2.len}`)
  // myHostFunc2(msg2.offset)
  // myHostFunc1(msg.offset)
  // myHostFunc2("Hello from JS2")

  Host.outputString(`Hello, World!`)
}

module.exports = { greet }
