function greet() {
  var pass = true;

  // Test basic array of objects
  console.table([
    { name: "Alice", age: 30 },
    { name: "Bob", age: 25 },
  ]);
  console.log("table array of objects: OK");

  // Test with column filter
  console.table(
    [
      { name: "Alice", age: 30, city: "NYC" },
      { name: "Bob", age: 25, city: "LA" },
    ],
    ["name", "city"],
  );
  console.log("table with columns: OK");

  // Test simple array (no object keys)
  console.table([1, 2, 3]);
  console.log("table simple array: OK");

  // Test object (non-array)
  console.table({ a: 1, b: 2, c: 3 });
  console.log("table plain object: OK");

  // Test empty array
  console.table([]);
  console.log("table empty array: OK");

  // Test non-object input (should just log it)
  console.table("hello");
  console.table(42);
  console.table(null);
  console.log("table non-object: OK");

  if (!pass) {
    throw new Error("console.table tests failed");
  }

  Host.outputString("console_table: all tests passed");
}

module.exports = { greet };
