function greet() {
  var pass = true;

  // Test primitives
  if (structuredClone(42) !== 42) { console.error("FAIL: number"); pass = false; }
  if (structuredClone("hello") !== "hello") { console.error("FAIL: string"); pass = false; }
  if (structuredClone(true) !== true) { console.error("FAIL: boolean"); pass = false; }
  if (structuredClone(null) !== null) { console.error("FAIL: null"); pass = false; }
  if (structuredClone(undefined) !== undefined) { console.error("FAIL: undefined"); pass = false; }
  console.log("primitives: OK");

  // Test plain object (deep copy)
  var obj = { a: 1, b: { c: 2 } };
  var cloned = structuredClone(obj);
  cloned.b.c = 99;
  if (obj.b.c === 2) {
    console.log("object deep copy: OK");
  } else {
    console.error("FAIL: object was shallow copied, obj.b.c =", obj.b.c);
    pass = false;
  }

  // Test array (deep copy)
  var arr = [1, [2, 3], { x: 4 }];
  var clonedArr = structuredClone(arr);
  clonedArr[1][0] = 99;
  clonedArr[2].x = 99;
  if (arr[1][0] === 2 && arr[2].x === 4) {
    console.log("array deep copy: OK");
  } else {
    console.error("FAIL: array was shallow copied");
    pass = false;
  }

  // Test Date
  var date = new Date(2024, 0, 15);
  var clonedDate = structuredClone(date);
  if (clonedDate instanceof Date && clonedDate.getTime() === date.getTime()) {
    console.log("Date clone: OK");
  } else {
    console.error("FAIL: Date clone");
    pass = false;
  }

  // Test RegExp
  var regex = new RegExp("hello", "gi");
  var clonedRegex = structuredClone(regex);
  if (clonedRegex instanceof RegExp && clonedRegex.source === "hello" && clonedRegex.flags === "gi") {
    console.log("RegExp clone: OK");
  } else {
    console.error("FAIL: RegExp clone");
    pass = false;
  }

  // Test Map
  var map = new Map();
  map.set("key1", "value1");
  map.set("key2", { nested: true });
  var clonedMap = structuredClone(map);
  clonedMap.get("key2").nested = false;
  if (map.get("key2").nested === true && clonedMap.size === 2) {
    console.log("Map clone: OK");
  } else {
    console.error("FAIL: Map clone");
    pass = false;
  }

  // Test Set
  var set = new Set();
  set.add(1);
  set.add(2);
  set.add(3);
  var clonedSet = structuredClone(set);
  if (clonedSet instanceof Set && clonedSet.size === 3 && clonedSet.has(1)) {
    console.log("Set clone: OK");
  } else {
    console.error("FAIL: Set clone");
    pass = false;
  }

  // Test ArrayBuffer
  var buffer = new ArrayBuffer(4);
  var view = new Uint8Array(buffer);
  view[0] = 1; view[1] = 2; view[2] = 3; view[3] = 4;
  var clonedBuffer = structuredClone(buffer);
  var clonedView = new Uint8Array(clonedBuffer);
  clonedView[0] = 99;
  if (view[0] === 1 && clonedView[0] === 99) {
    console.log("ArrayBuffer clone: OK");
  } else {
    console.error("FAIL: ArrayBuffer clone");
    pass = false;
  }

  // Test circular reference
  var circular = { name: "root" };
  circular.self = circular;
  var clonedCircular = structuredClone(circular);
  if (clonedCircular.name === "root" && clonedCircular.self === clonedCircular) {
    console.log("circular reference: OK");
  } else {
    console.error("FAIL: circular reference");
    pass = false;
  }

  // Test Error
  var err = new Error("test error");
  var clonedErr = structuredClone(err);
  if (clonedErr instanceof Error && clonedErr.message === "test error") {
    console.log("Error clone: OK");
  } else {
    console.error("FAIL: Error clone");
    pass = false;
  }

  if (!pass) {
    throw new Error("structuredClone tests failed");
  }

  Host.outputString("structured_clone: all tests passed");
}

module.exports = { greet };
