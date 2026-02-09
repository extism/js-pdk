function greet() {
  // Test console.assert - should only log on failure
  console.assert(true, "this should NOT appear");
  console.assert(false, "assertion failed message");
  console.assert(false);
  console.log("console.assert: OK");

  // Test console.count
  console.count("myCounter");
  console.count("myCounter");
  console.count("myCounter");
  console.count();
  console.count();
  console.log("console.count: OK");

  // Test console.countReset
  console.countReset("myCounter");
  console.count("myCounter");
  console.log("console.countReset: OK (counter restarted at 1)");

  // Test console.countReset on unknown label warns
  console.countReset("unknownLabel");

  // Test console.time / console.timeEnd
  console.time("myTimer");
  // do a tiny bit of work
  var sum = 0;
  for (var i = 0; i < 1000; i++) { sum += i; }
  console.timeEnd("myTimer");
  console.log("console.time/timeEnd: OK");

  // Test console.time duplicate warning
  console.time("dupTimer");
  console.time("dupTimer");
  console.timeEnd("dupTimer");

  // Test console.timeEnd on unknown timer
  console.timeEnd("unknownTimer");

  // Test console.timeLog
  console.time("logTimer");
  console.timeLog("logTimer", "checkpoint 1");
  console.timeLog("logTimer", "checkpoint 2", { extra: "data" });
  console.timeEnd("logTimer");
  console.log("console.timeLog: OK");

  // Test console.timeLog on unknown timer
  console.timeLog("unknownTimer");

  Host.outputString("console_extra: all tests passed");
}

module.exports = { greet };
