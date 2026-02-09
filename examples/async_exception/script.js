// Reproducer for https://github.com/extism/js-pdk/issues/134
// An async function returns a Promise. If the caller doesn't await it,
// calling a method on the Promise (instead of the resolved value) throws
// TypeError. Previously this caused a panic (wasm error: unreachable)
// because execute_pending_job() corrupted the exception state.

async function createObject(opts) {
  return {
    doSomething: function(input) {
      return "processed: " + input;
    }
  };
}

function greet() {
  // Not awaited -- result is a Promise, not the resolved object
  var result = createObject({ setting: true });
  // Calling a method on the Promise throws TypeError: not a function
  var output = result.doSomething("hello");
  return 0;
}

module.exports = { greet };
