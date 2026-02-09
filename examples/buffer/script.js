function greet() {
  const results = [];

  // --- Creation ---
  // Buffer.from(string, encoding)
  const b1 = Buffer.from("hello");
  results.push(b1.toString() === "hello" ? "OK" : "FAIL");
  console.log("from string utf8: " + (results[results.length - 1]));

  const b2 = Buffer.from("68656c6c6f", "hex");
  results.push(b2.toString() === "hello" ? "OK" : "FAIL");
  console.log("from hex: " + (results[results.length - 1]));

  const b3 = Buffer.from("aGVsbG8=", "base64");
  results.push(b3.toString() === "hello" ? "OK" : "FAIL");
  console.log("from base64: " + (results[results.length - 1]));

  const b3u = Buffer.from("aGVsbG8", "base64url");
  results.push(b3u.toString() === "hello" ? "OK" : "FAIL");
  console.log("from base64url: " + (results[results.length - 1]));

  const b4 = Buffer.from([0x48, 0x69]);
  results.push(b4.toString() === "Hi" ? "OK" : "FAIL");
  console.log("from array: " + (results[results.length - 1]));

  // Buffer.from(buffer) copies
  const b5 = Buffer.from(b1);
  b5[0] = 0x48;
  results.push(b1[0] !== 0x48 && b5.toString() === "Hello" ? "OK" : "FAIL");
  console.log("from buffer copies: " + (results[results.length - 1]));

  // Buffer.alloc
  const b6 = Buffer.alloc(5, 0x41);
  results.push(b6.toString() === "AAAAA" ? "OK" : "FAIL");
  console.log("alloc with fill: " + (results[results.length - 1]));

  const b7 = Buffer.allocUnsafe(4);
  results.push(b7.length === 4 ? "OK" : "FAIL");
  console.log("allocUnsafe: " + (results[results.length - 1]));

  // --- Static methods ---
  results.push(Buffer.isBuffer(b1) === true && Buffer.isBuffer("nope") === false ? "OK" : "FAIL");
  console.log("isBuffer: " + (results[results.length - 1]));

  results.push(Buffer.isEncoding("utf8") === true && Buffer.isEncoding("nope") === false ? "OK" : "FAIL");
  console.log("isEncoding: " + (results[results.length - 1]));

  results.push(Buffer.byteLength("hello") === 5 ? "OK" : "FAIL");
  console.log("byteLength: " + (results[results.length - 1]));

  // byteLength for multi-byte utf8
  results.push(Buffer.byteLength("\u00e9") === 2 ? "OK" : "FAIL");
  console.log("byteLength multibyte: " + (results[results.length - 1]));

  const c1 = Buffer.from("hel");
  const c2 = Buffer.from("lo");
  const c3 = Buffer.concat([c1, c2]);
  results.push(c3.toString() === "hello" && c3.length === 5 ? "OK" : "FAIL");
  console.log("concat: " + (results[results.length - 1]));

  // concat with totalLength truncation
  const c4 = Buffer.concat([c1, c2], 3);
  results.push(c4.toString() === "hel" && c4.length === 3 ? "OK" : "FAIL");
  console.log("concat truncated: " + (results[results.length - 1]));

  results.push(Buffer.compare(Buffer.from("a"), Buffer.from("b")) === -1 ? "OK" : "FAIL");
  console.log("static compare: " + (results[results.length - 1]));

  // --- Encoding roundtrips ---
  const orig = Buffer.from("Hello, World!");
  results.push(orig.toString("hex") === "48656c6c6f2c20576f726c6421" ? "OK" : "FAIL");
  console.log("toString hex: " + (results[results.length - 1]));

  results.push(orig.toString("base64") === "SGVsbG8sIFdvcmxkIQ==" ? "OK" : "FAIL");
  console.log("toString base64: " + (results[results.length - 1]));

  results.push(orig.toString("base64url") === "SGVsbG8sIFdvcmxkIQ" ? "OK" : "FAIL");
  console.log("toString base64url: " + (results[results.length - 1]));

  results.push(orig.toString("latin1") === "Hello, World!" ? "OK" : "FAIL");
  console.log("toString latin1: " + (results[results.length - 1]));

  // toString with start/end
  results.push(orig.toString("utf8", 0, 5) === "Hello" ? "OK" : "FAIL");
  console.log("toString slice: " + (results[results.length - 1]));

  // --- Instance methods ---
  // slice shares memory
  const sliceSrc = Buffer.from("abcdef");
  const sliced = sliceSrc.slice(1, 4);
  results.push(sliced.toString() === "bcd" ? "OK" : "FAIL");
  console.log("slice: " + (results[results.length - 1]));

  sliced[0] = 0x42; // 'B'
  results.push(sliceSrc[1] === 0x42 ? "OK" : "FAIL");
  console.log("slice shares memory: " + (results[results.length - 1]));

  // copy
  const copySrc = Buffer.from("hello");
  const copyDst = Buffer.alloc(5);
  copySrc.copy(copyDst);
  results.push(copyDst.toString() === "hello" ? "OK" : "FAIL");
  console.log("copy: " + (results[results.length - 1]));

  // copy with offsets
  const copyDst2 = Buffer.alloc(10, 0x2e);
  copySrc.copy(copyDst2, 3);
  results.push(copyDst2.toString() === "...hello.." ? "OK" : "FAIL");
  console.log("copy with offset: " + (results[results.length - 1]));

  // write
  const writeBuf = Buffer.alloc(8);
  writeBuf.write("Hi", 0);
  writeBuf.write("!!!", 2);
  results.push(writeBuf.toString("utf8", 0, 5) === "Hi!!!" ? "OK" : "FAIL");
  console.log("write: " + (results[results.length - 1]));

  // equals
  results.push(Buffer.from("abc").equals(Buffer.from("abc")) === true ? "OK" : "FAIL");
  console.log("equals true: " + (results[results.length - 1]));

  results.push(Buffer.from("abc").equals(Buffer.from("def")) === false ? "OK" : "FAIL");
  console.log("equals false: " + (results[results.length - 1]));

  // instance compare
  results.push(Buffer.from("abc").compare(Buffer.from("abc")) === 0 ? "OK" : "FAIL");
  console.log("compare equal: " + (results[results.length - 1]));

  results.push(Buffer.from("abc").compare(Buffer.from("abd")) === -1 ? "OK" : "FAIL");
  console.log("compare less: " + (results[results.length - 1]));

  // indexOf
  const haystack = Buffer.from("hello world hello");
  results.push(haystack.indexOf("world") === 6 ? "OK" : "FAIL");
  console.log("indexOf string: " + (results[results.length - 1]));

  results.push(haystack.indexOf(0x6f) === 4 ? "OK" : "FAIL");
  console.log("indexOf byte: " + (results[results.length - 1]));

  results.push(haystack.indexOf("hello", 1) === 12 ? "OK" : "FAIL");
  console.log("indexOf with offset: " + (results[results.length - 1]));

  // includes
  results.push(haystack.includes("world") === true ? "OK" : "FAIL");
  console.log("includes true: " + (results[results.length - 1]));

  results.push(haystack.includes("xyz") === false ? "OK" : "FAIL");
  console.log("includes false: " + (results[results.length - 1]));

  // fill
  const fillBuf = Buffer.alloc(6);
  fillBuf.fill("ab");
  results.push(fillBuf.toString() === "ababab" ? "OK" : "FAIL");
  console.log("fill string: " + (results[results.length - 1]));

  // toJSON
  const jsonBuf = Buffer.from([1, 2, 3]);
  const json = jsonBuf.toJSON();
  results.push(json.type === "Buffer" && json.data[0] === 1 && json.data[2] === 3 ? "OK" : "FAIL");
  console.log("toJSON: " + (results[results.length - 1]));

  // toJSON roundtrip
  const roundtrip = Buffer.from(json);
  results.push(roundtrip.equals(jsonBuf) ? "OK" : "FAIL");
  console.log("toJSON roundtrip: " + (results[results.length - 1]));

  // --- Read/write integers ---
  const intBuf = Buffer.alloc(8);
  intBuf.writeUInt8(0xff, 0);
  results.push(intBuf.readUInt8(0) === 255 ? "OK" : "FAIL");
  console.log("readUInt8/writeUInt8: " + (results[results.length - 1]));

  intBuf.writeUInt16BE(0x0102, 0);
  results.push(intBuf.readUInt16BE(0) === 0x0102 ? "OK" : "FAIL");
  console.log("readUInt16BE/writeUInt16BE: " + (results[results.length - 1]));

  intBuf.writeUInt16LE(0x0304, 2);
  results.push(intBuf.readUInt16LE(2) === 0x0304 ? "OK" : "FAIL");
  console.log("readUInt16LE/writeUInt16LE: " + (results[results.length - 1]));

  intBuf.writeUInt32BE(0xDEADBEEF, 0);
  results.push(intBuf.readUInt32BE(0) === 0xDEADBEEF ? "OK" : "FAIL");
  console.log("readUInt32BE/writeUInt32BE: " + (results[results.length - 1]));

  intBuf.writeUInt32LE(0xCAFEBABE, 4);
  results.push(intBuf.readUInt32LE(4) === 0xCAFEBABE ? "OK" : "FAIL");
  console.log("readUInt32LE/writeUInt32LE: " + (results[results.length - 1]));

  intBuf.writeInt8(-42, 0);
  results.push(intBuf.readInt8(0) === -42 ? "OK" : "FAIL");
  console.log("readInt8/writeInt8: " + (results[results.length - 1]));

  intBuf.writeInt16BE(-1000, 0);
  results.push(intBuf.readInt16BE(0) === -1000 ? "OK" : "FAIL");
  console.log("readInt16BE/writeInt16BE: " + (results[results.length - 1]));

  intBuf.writeInt32LE(-123456, 0);
  results.push(intBuf.readInt32LE(0) === -123456 ? "OK" : "FAIL");
  console.log("readInt32LE/writeInt32LE: " + (results[results.length - 1]));

  // Float/Double
  const floatBuf = Buffer.alloc(12);
  floatBuf.writeFloatBE(3.14, 0);
  results.push(Math.abs(floatBuf.readFloatBE(0) - 3.14) < 0.001 ? "OK" : "FAIL");
  console.log("readFloatBE/writeFloatBE: " + (results[results.length - 1]));

  floatBuf.writeDoubleLE(Math.PI, 0);
  results.push(floatBuf.readDoubleLE(0) === Math.PI ? "OK" : "FAIL");
  console.log("readDoubleLE/writeDoubleLE: " + (results[results.length - 1]));

  // --- Length and instanceof ---
  results.push(Buffer.from("test").length === 4 ? "OK" : "FAIL");
  console.log("length: " + (results[results.length - 1]));

  results.push(Buffer.from("test") instanceof Uint8Array ? "OK" : "FAIL");
  console.log("instanceof Uint8Array: " + (results[results.length - 1]));

  const allPassed = results.every(function(r) { return r === "OK"; });
  Host.outputString("buffer: " + (allPassed ? "all tests passed" : "FAILED"));
}

module.exports = { greet };
