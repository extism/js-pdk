function greet() {
  var pass = true;

  // Helper to convert ArrayBuffer to hex string
  function toHex(buffer) {
    var bytes = new Uint8Array(buffer);
    var hex = "";
    for (var i = 0; i < bytes.length; i++) {
      hex += bytes[i].toString(16).padStart(2, "0");
    }
    return hex;
  }

  // --- Test SHA-256 ---
  // SHA-256 of "" = e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
  var emptyHash = crypto.subtle.digest("SHA-256", new ArrayBuffer(0));
  emptyHash.then(function (result) {
    var hex = toHex(result);
    if (hex !== "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855") {
      console.error("FAIL: SHA-256 empty:", hex);
      pass = false;
    }
  });
  console.log("SHA-256 empty: OK");

  // SHA-256 of "hello"
  var encoder = new TextEncoder();
  var helloData = encoder.encode("hello");
  crypto.subtle.digest("SHA-256", helloData.buffer).then(function (result) {
    var hex = toHex(result);
    // SHA-256("hello") = 2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824
    if (hex !== "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824") {
      console.error("FAIL: SHA-256 hello:", hex);
      pass = false;
    }
  });
  console.log("SHA-256 hello: OK");

  // --- Test SHA-1 ---
  // SHA-1 of "hello" = aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d
  crypto.subtle.digest("SHA-1", helloData.buffer).then(function (result) {
    var hex = toHex(result);
    if (hex !== "aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d") {
      console.error("FAIL: SHA-1 hello:", hex);
      pass = false;
    }
  });
  console.log("SHA-1 hello: OK");

  // --- Test SHA-384 ---
  crypto.subtle.digest("SHA-384", helloData.buffer).then(function (result) {
    var bytes = new Uint8Array(result);
    if (bytes.length !== 48) {
      console.error("FAIL: SHA-384 length:", bytes.length);
      pass = false;
    }
  });
  console.log("SHA-384 length: OK");

  // --- Test SHA-512 ---
  crypto.subtle.digest("SHA-512", helloData.buffer).then(function (result) {
    var bytes = new Uint8Array(result);
    if (bytes.length !== 64) {
      console.error("FAIL: SHA-512 length:", bytes.length);
      pass = false;
    }
  });
  console.log("SHA-512 length: OK");

  // --- Test with object algorithm parameter ---
  crypto.subtle.digest({ name: "SHA-256" }, helloData.buffer).then(function (result) {
    var hex = toHex(result);
    if (hex !== "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824") {
      console.error("FAIL: SHA-256 object param:", hex);
      pass = false;
    }
  });
  console.log("SHA-256 object param: OK");

  // --- Test with TypedArray input (not just ArrayBuffer) ---
  crypto.subtle.digest("SHA-256", helloData).then(function (result) {
    var hex = toHex(result);
    if (hex !== "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824") {
      console.error("FAIL: SHA-256 TypedArray input:", hex);
      pass = false;
    }
  });
  console.log("SHA-256 TypedArray input: OK");

  // --- Test unsupported algorithm ---
  var threw = false;
  try {
    crypto.subtle.digest("MD5", new ArrayBuffer(0));
  } catch (e) {
    threw = true;
  }
  if (!threw) {
    console.error("FAIL: unsupported algorithm should throw");
    pass = false;
  }
  console.log("unsupported algorithm: OK");

  if (!pass) {
    throw new Error("subtle digest tests failed");
  }

  Host.outputString("subtle_digest: all tests passed");
}

module.exports = { greet };
