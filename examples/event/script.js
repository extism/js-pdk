function greet() {
  var pass = true;

  // --- Test Event constructor ---
  var ev = new Event("click");
  if (ev.type !== "click") {
    console.error("FAIL: Event.type");
    pass = false;
  }
  if (ev.bubbles !== false) {
    console.error("FAIL: Event.bubbles default");
    pass = false;
  }
  if (ev.cancelable !== false) {
    console.error("FAIL: Event.cancelable default");
    pass = false;
  }
  if (typeof ev.timeStamp !== "number") {
    console.error("FAIL: Event.timeStamp");
    pass = false;
  }
  console.log("Event constructor: OK");

  // --- Test Event with options ---
  var ev2 = new Event("submit", { bubbles: true, cancelable: true });
  if (!ev2.bubbles || !ev2.cancelable) {
    console.error("FAIL: Event options");
    pass = false;
  }
  console.log("Event options: OK");

  // --- Test preventDefault ---
  ev2.preventDefault();
  if (!ev2.defaultPrevented) {
    console.error("FAIL: preventDefault");
    pass = false;
  }
  // Non-cancelable event should not be preventable
  var ev3 = new Event("test", { cancelable: false });
  ev3.preventDefault();
  if (ev3.defaultPrevented) {
    console.error("FAIL: non-cancelable preventDefault");
    pass = false;
  }
  console.log("preventDefault: OK");

  // --- Test EventTarget basic usage ---
  var target = new EventTarget();
  var callCount = 0;
  var handler = function (e) {
    callCount++;
    if (e.type !== "test") {
      console.error("FAIL: event type in handler");
      pass = false;
    }
    if (e.target !== target) {
      console.error("FAIL: event target in handler");
      pass = false;
    }
  };

  target.addEventListener("test", handler);
  target.dispatchEvent(new Event("test"));

  if (callCount !== 1) {
    console.error("FAIL: handler called", callCount, "times");
    pass = false;
  }
  console.log("EventTarget basic: OK");

  // --- Test multiple listeners ---
  var calls = [];
  target.addEventListener("multi", function () { calls.push("a"); });
  target.addEventListener("multi", function () { calls.push("b"); });
  target.dispatchEvent(new Event("multi"));
  if (calls.join(",") !== "a,b") {
    console.error("FAIL: multiple listeners:", calls.join(","));
    pass = false;
  }
  console.log("EventTarget multiple listeners: OK");

  // --- Test removeEventListener ---
  callCount = 0;
  target.removeEventListener("test", handler);
  target.dispatchEvent(new Event("test"));
  if (callCount !== 0) {
    console.error("FAIL: handler still called after remove");
    pass = false;
  }
  console.log("removeEventListener: OK");

  // --- Test once option ---
  var onceCount = 0;
  target.addEventListener("once-test", function () { onceCount++; }, { once: true });
  target.dispatchEvent(new Event("once-test"));
  target.dispatchEvent(new Event("once-test"));
  if (onceCount !== 1) {
    console.error("FAIL: once listener called", onceCount, "times");
    pass = false;
  }
  console.log("addEventListener once: OK");

  // --- Test dispatchEvent return value ---
  var target2 = new EventTarget();
  target2.addEventListener("cancel", function (e) { e.preventDefault(); });
  var result = target2.dispatchEvent(new Event("cancel", { cancelable: true }));
  if (result !== false) {
    console.error("FAIL: dispatchEvent should return false when prevented");
    pass = false;
  }
  var result2 = target2.dispatchEvent(new Event("other"));
  if (result2 !== true) {
    console.error("FAIL: dispatchEvent should return true when not prevented");
    pass = false;
  }
  console.log("dispatchEvent return: OK");

  // --- Test no duplicate listeners ---
  var dupCount = 0;
  var dupHandler = function () { dupCount++; };
  var target3 = new EventTarget();
  target3.addEventListener("dup", dupHandler);
  target3.addEventListener("dup", dupHandler);
  target3.dispatchEvent(new Event("dup"));
  if (dupCount !== 1) {
    console.error("FAIL: duplicate listener added, count:", dupCount);
    pass = false;
  }
  console.log("no duplicate listeners: OK");

  // --- Test stopImmediatePropagation ---
  var sipCalls = [];
  var target4 = new EventTarget();
  target4.addEventListener("sip", function (e) { sipCalls.push("first"); e.stopImmediatePropagation(); });
  target4.addEventListener("sip", function () { sipCalls.push("second"); });
  target4.dispatchEvent(new Event("sip"));
  if (sipCalls.join(",") !== "first") {
    console.error("FAIL: stopImmediatePropagation:", sipCalls.join(","));
    pass = false;
  }
  console.log("stopImmediatePropagation: OK");

  if (!pass) {
    throw new Error("event tests failed");
  }

  Host.outputString("event: all tests passed");
}

module.exports = { greet };
