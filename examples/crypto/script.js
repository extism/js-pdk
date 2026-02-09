function greet() {
  var pass = true;

  // --- Test crypto.getRandomValues with Uint8Array ---
  var arr = new Uint8Array(16);
  var result = crypto.getRandomValues(arr);

  // Should return the same array
  if (result !== arr) {
    console.error("FAIL: getRandomValues should return the same array");
    pass = false;
  }

  // Should have filled with non-zero bytes (statistically near-impossible for all 16 to be zero)
  var allZero = true;
  for (var i = 0; i < arr.length; i++) {
    if (arr[i] !== 0) {
      allZero = false;
      break;
    }
  }
  if (allZero) {
    console.error("FAIL: getRandomValues produced all zeros");
    pass = false;
  }
  console.log("getRandomValues Uint8Array: OK");

  // --- Test with Uint32Array ---
  var arr32 = new Uint32Array(4);
  crypto.getRandomValues(arr32);
  var allZero32 = true;
  for (var i = 0; i < arr32.length; i++) {
    if (arr32[i] !== 0) {
      allZero32 = false;
      break;
    }
  }
  if (allZero32) {
    console.error("FAIL: getRandomValues Uint32Array all zeros");
    pass = false;
  }
  console.log("getRandomValues Uint32Array: OK");

  // --- Test with Int16Array ---
  var arr16 = new Int16Array(8);
  crypto.getRandomValues(arr16);
  console.log("getRandomValues Int16Array: OK");

  // --- Test two calls produce different values (with very high probability) ---
  var a = new Uint8Array(32);
  var b = new Uint8Array(32);
  crypto.getRandomValues(a);
  crypto.getRandomValues(b);
  var same = true;
  for (var i = 0; i < a.length; i++) {
    if (a[i] !== b[i]) {
      same = false;
      break;
    }
  }
  if (same) {
    console.error("FAIL: two getRandomValues calls produced identical output");
    pass = false;
  }
  console.log("getRandomValues uniqueness: OK");

  // --- Test quota limit (>65536 bytes should throw) ---
  var threw = false;
  try {
    crypto.getRandomValues(new Uint8Array(65537));
  } catch (e) {
    threw = true;
  }
  if (!threw) {
    console.error("FAIL: getRandomValues should throw for >65536 bytes");
    pass = false;
  }
  console.log("getRandomValues quota limit: OK");

  // --- Test randomUUID format ---
  var uuid = crypto.randomUUID();
  console.log("randomUUID:", uuid);
  // UUID v4 format: xxxxxxxx-xxxx-4xxx-[89ab]xxx-xxxxxxxxxxxx
  var uuidRegex = /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/;
  if (!uuidRegex.test(uuid)) {
    console.error("FAIL: randomUUID format invalid:", uuid);
    pass = false;
  }
  console.log("randomUUID format: OK");

  // --- Test randomUUID uniqueness ---
  var uuid2 = crypto.randomUUID();
  if (uuid === uuid2) {
    console.error("FAIL: two randomUUID calls produced same value");
    pass = false;
  }
  console.log("randomUUID uniqueness: OK");

  // --- Test randomUUID length ---
  if (uuid.length !== 36) {
    console.error("FAIL: randomUUID length should be 36, got:", uuid.length);
    pass = false;
  }
  console.log("randomUUID length: OK");

  if (!pass) {
    throw new Error("crypto tests failed");
  }

  Host.outputString("crypto: all tests passed");
}

module.exports = { greet };
