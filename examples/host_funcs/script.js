// Extract host functions by name.
// Note: these must be declared in the d.ts file
const { capitalize, floatInputs, floatOutput, voidInputs } = Host.getFunctions()

function greet() {
  const name = Host.inputString();
  const mem = Memory.fromString(name);
  const capNameOffset = capitalize(mem.offset);
  const capName = mem.readString(capNameOffset);
  console.log(`Hello, ${capName}!`);

  const f32 = 314_567.5;
  const i32 = 2_147_483_647;
  const f64 = 9_007_199_254_740.125;
  const i64 = 9_223_372_036_854_775_807;

  const num1 = floatInputs(f64, f32);
  console.log(`floatInputs result: ${num1}`);
  if (num1 != i32) {
    throw new Error(`Unexpected floatInputs result: ${num1}. Expected: ${i32}`);
  }

  const num2 = floatOutput(i32);
  console.log(`floatOutput result: ${num2}`);
  if (Math.abs(num2 - f64) >= 0.001) {
    throw new Error(`Unexpected floatOutput result: ${num2}. Expected: ${f64}`);
  }

  voidInputs(i32, i64, f32, f64, i32);

  console.log("All tests passed!");
  Host.outputString(`Hello, ${capName}!`);
}

module.exports = { greet }
