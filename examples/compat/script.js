function greet() {
  var pass = true;

  // --- Test globalThis.self ---
  if (self !== globalThis) {
    console.error("FAIL: self !== globalThis");
    pass = false;
  }
  if (typeof self === "undefined") {
    console.error("FAIL: self is undefined");
    pass = false;
  }
  console.log("self: OK");

  // --- Test queueMicrotask ---
  var microtaskRan = false;
  queueMicrotask(function () {
    microtaskRan = true;
  });
  // In our runtime, microtask executes immediately (synchronous)
  if (!microtaskRan) {
    console.error("FAIL: queueMicrotask did not run");
    pass = false;
  }
  console.log("queueMicrotask: OK");

  // --- Test queueMicrotask execution order ---
  var order = [];
  order.push("a");
  queueMicrotask(function () {
    order.push("b");
  });
  order.push("c");
  // In synchronous model: a, b, c (microtask runs immediately)
  if (order.join(",") !== "a,b,c") {
    console.error("FAIL: queueMicrotask order:", order.join(","));
    pass = false;
  }
  console.log("queueMicrotask order: OK");

  // --- Test DOMException ---
  var ex = new DOMException("test message", "NotFoundError");
  if (ex.name !== "NotFoundError") {
    console.error("FAIL: DOMException.name:", ex.name);
    pass = false;
  }
  if (ex.message !== "test message") {
    console.error("FAIL: DOMException.message:", ex.message);
    pass = false;
  }
  if (ex.code !== 8) {
    console.error("FAIL: DOMException.code:", ex.code);
    pass = false;
  }
  if (!(ex instanceof Error)) {
    console.error("FAIL: DOMException not instanceof Error");
    pass = false;
  }
  if (!(ex instanceof DOMException)) {
    console.error("FAIL: DOMException not instanceof DOMException");
    pass = false;
  }
  console.log("DOMException: OK");

  // --- Test DOMException from atob ---
  var caught = false;
  try {
    atob("!");  // invalid base64
  } catch (e) {
    caught = true;
    if (!(e instanceof DOMException)) {
      console.error("FAIL: atob error not DOMException");
      pass = false;
    }
    if (e.name !== "InvalidCharacterError") {
      console.error("FAIL: atob error name:", e.name);
      pass = false;
    }
  }
  if (!caught) {
    console.error("FAIL: atob did not throw");
    pass = false;
  }
  console.log("DOMException from atob: OK");

  if (!pass) {
    throw new Error("compat tests failed");
  }

  Host.outputString("compat: all tests passed");
}

module.exports = { greet };
