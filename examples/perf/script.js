function greet() {
  var pass = true;

  // Test performance.timeOrigin is a number
  if (typeof performance.timeOrigin !== "number" || isNaN(performance.timeOrigin)) {
    console.error("FAIL: performance.timeOrigin is not a number:", performance.timeOrigin);
    pass = false;
  } else {
    console.log("performance.timeOrigin:", performance.timeOrigin);
  }

  // Test performance.timeOrigin is a reasonable epoch ms (after year 2020)
  if (performance.timeOrigin < 1577836800000) {
    console.error("FAIL: performance.timeOrigin seems too small:", performance.timeOrigin);
    pass = false;
  }

  // Test performance.now() returns a number
  var t1 = performance.now();
  if (typeof t1 !== "number" || isNaN(t1)) {
    console.error("FAIL: performance.now() is not a number:", t1);
    pass = false;
  } else {
    console.log("performance.now():", t1);
  }

  // Test performance.now() is >= 0 (time since module init)
  if (t1 < 0) {
    console.error("FAIL: performance.now() is negative:", t1);
    pass = false;
  }

  // Test sequential calls are monotonic
  var t2 = performance.now();
  if (t2 < t1) {
    console.error("FAIL: performance.now() is not monotonic:", t1, "->", t2);
    pass = false;
  } else {
    console.log("monotonic check: OK (", t1, "<=", t2, ")");
  }

  // Do some work and measure
  var sum = 0;
  var start = performance.now();
  for (var i = 0; i < 10000; i++) {
    sum += i;
  }
  var elapsed = performance.now() - start;
  console.log("loop elapsed:", elapsed, "ms (sum:", sum, ")");
  if (typeof elapsed !== "number") {
    console.error("FAIL: elapsed is not a number");
    pass = false;
  }

  if (!pass) {
    throw new Error("performance tests failed");
  }

  Host.outputString("performance: all tests passed");
}

module.exports = { greet };
