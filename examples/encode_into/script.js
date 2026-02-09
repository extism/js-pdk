function greet() {
  var pass = true;
  var encoder = new TextEncoder();

  // Test basic ASCII encode into
  var dest1 = new Uint8Array(20);
  var result1 = encoder.encodeInto("Hello", dest1);
  console.log("encodeInto('Hello'):", JSON.stringify(result1));
  if (result1.read !== 5 || result1.written !== 5) {
    console.error("FAIL: ASCII encodeInto wrong result");
    pass = false;
  }
  // Verify bytes
  var expected = [72, 101, 108, 108, 111]; // H, e, l, l, o
  for (var i = 0; i < expected.length; i++) {
    if (dest1[i] !== expected[i]) {
      console.error("FAIL: byte", i, "expected", expected[i], "got", dest1[i]);
      pass = false;
    }
  }
  console.log("ASCII encodeInto: OK");

  // Test with destination too small - should truncate
  var dest2 = new Uint8Array(3);
  var result2 = encoder.encodeInto("Hello", dest2);
  console.log("encodeInto('Hello', [3]):", JSON.stringify(result2));
  if (result2.written !== 3 || result2.read !== 3) {
    console.error("FAIL: truncated encodeInto wrong result");
    pass = false;
  }
  console.log("truncated encodeInto: OK");

  // Test with multi-byte UTF-8 characters
  var dest3 = new Uint8Array(20);
  var result3 = encoder.encodeInto("\u00e9", dest3); // e-acute: 2 bytes in UTF-8
  console.log("encodeInto('e-acute'):", JSON.stringify(result3));
  if (result3.written !== 2) {
    console.error("FAIL: multi-byte character written count wrong, got:", result3.written);
    pass = false;
  }
  console.log("multi-byte encodeInto: OK");

  // Test empty string
  var dest4 = new Uint8Array(10);
  var result4 = encoder.encodeInto("", dest4);
  console.log("encodeInto(''):", JSON.stringify(result4));
  if (result4.read !== 0 || result4.written !== 0) {
    console.error("FAIL: empty string encodeInto wrong result");
    pass = false;
  }
  console.log("empty encodeInto: OK");

  // Test zero-length destination
  var dest5 = new Uint8Array(0);
  var result5 = encoder.encodeInto("Hello", dest5);
  console.log("encodeInto into [0]:", JSON.stringify(result5));
  if (result5.read !== 0 || result5.written !== 0) {
    console.error("FAIL: zero-length dest encodeInto wrong result");
    pass = false;
  }
  console.log("zero-length dest encodeInto: OK");

  // Test roundtrip: encodeInto then decode
  var source = "Hello, World!";
  var dest6 = new Uint8Array(50);
  var result6 = encoder.encodeInto(source, dest6);
  var decoded = new TextDecoder().decode(dest6.subarray(0, result6.written));
  if (decoded !== source) {
    console.error("FAIL: roundtrip mismatch:", decoded, "!==", source);
    pass = false;
  }
  console.log("roundtrip: OK");

  if (!pass) {
    throw new Error("encodeInto tests failed");
  }

  Host.outputString("encode_into: all tests passed");
}

module.exports = { greet };
