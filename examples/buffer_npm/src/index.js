// Test that an npm package which uses require('buffer') works with our Buffer polyfill
var crc32 = require("buffer-crc32");

function greet() {
  var results = [];

  // Test 1: crc32.unsigned with string input returns a number
  var r1 = crc32.unsigned("hello");
  results.push(typeof r1 === "number" ? "OK" : "FAIL");
  console.log("crc32 string input: " + results[results.length - 1]);

  // Test 2: crc32.unsigned with Buffer input works
  var buf = Buffer.from("hello");
  var r2 = crc32.unsigned(buf);
  results.push(typeof r2 === "number" ? "OK" : "FAIL");
  console.log("crc32 buffer input: " + results[results.length - 1]);

  // Test 3: String and Buffer inputs produce the same CRC32
  results.push(r1 === r2 ? "OK" : "FAIL");
  console.log("string equals buffer: " + results[results.length - 1]);

  // Test 4: Different inputs produce different CRC32s
  var r3 = crc32.unsigned("world");
  results.push(r1 !== r3 ? "OK" : "FAIL");
  console.log("different inputs differ: " + results[results.length - 1]);

  // Test 5: Same input is deterministic
  var r4 = crc32.unsigned("hello");
  results.push(r1 === r4 ? "OK" : "FAIL");
  console.log("deterministic: " + results[results.length - 1]);

  // Test 6: crc32() returns a Buffer
  var checksumBuf = crc32("hello");
  results.push(Buffer.isBuffer(checksumBuf) ? "OK" : "FAIL");
  console.log("returns buffer: " + results[results.length - 1]);

  // Test 7: The returned Buffer has 4 bytes (32 bits)
  results.push(checksumBuf.length === 4 ? "OK" : "FAIL");
  console.log("buffer length 4: " + results[results.length - 1]);

  // Test 8: Can read the CRC32 as a UInt32
  var asUint = checksumBuf.readUInt32BE(0);
  results.push(typeof asUint === "number" && asUint > 0 ? "OK" : "FAIL");
  console.log("readUInt32BE: " + results[results.length - 1]);

  var allPassed = results.every(function (r) {
    return r === "OK";
  });
  Host.outputString(
    "buffer_npm: " + (allPassed ? "all tests passed" : "FAILED"),
  );
}

module.exports = { greet };
