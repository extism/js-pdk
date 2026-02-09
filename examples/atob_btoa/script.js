function greet() {
  var pass = true;

  // Test btoa: encode ASCII string to base64
  var encoded = btoa("Hello, World!");
  console.log("btoa('Hello, World!') =", encoded);
  if (encoded !== "SGVsbG8sIFdvcmxkIQ==") {
    console.error("FAIL: btoa produced wrong output:", encoded);
    pass = false;
  }

  // Test atob: decode base64 back to ASCII
  var decoded = atob("SGVsbG8sIFdvcmxkIQ==");
  console.log("atob('SGVsbG8sIFdvcmxkIQ==') =", decoded);
  if (decoded !== "Hello, World!") {
    console.error("FAIL: atob produced wrong output:", decoded);
    pass = false;
  }

  // Test roundtrip
  var original = "The quick brown fox jumps over the lazy dog";
  var roundtrip = atob(btoa(original));
  console.log("roundtrip =", roundtrip);
  if (roundtrip !== original) {
    console.error("FAIL: roundtrip mismatch");
    pass = false;
  }

  // Test binary data (Latin1 range)
  var binaryStr = "";
  for (var i = 0; i < 256; i++) {
    binaryStr += String.fromCharCode(i);
  }
  var binaryEncoded = btoa(binaryStr);
  var binaryDecoded = atob(binaryEncoded);
  if (binaryDecoded !== binaryStr) {
    console.error("FAIL: binary roundtrip mismatch");
    pass = false;
  }
  console.log("binary roundtrip: OK (256 chars)");

  // Test btoa throws on non-Latin1 characters
  try {
    btoa("\u0100");
    console.error("FAIL: btoa should throw on non-Latin1");
    pass = false;
  } catch (e) {
    console.log("btoa non-Latin1 throws:", e.message);
  }

  // Test atob throws on invalid input
  try {
    atob("A");
    console.error("FAIL: atob should throw on invalid base64");
    pass = false;
  } catch (e) {
    console.log("atob invalid throws:", e.message);
  }

  if (!pass) {
    throw new Error("atob/btoa tests failed");
  }

  Host.outputString("atob_btoa: all tests passed");
}

module.exports = { greet };
