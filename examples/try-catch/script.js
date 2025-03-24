function greet() {
    callListeners();
}
  
const LISTENERS = {
  hello: async () => {
    console.log("inside closure");
    throw new Error("hello world");
  },
};

async function callListeners() {
  try {
    await LISTENERS["hello"]();
  } catch (error) {
    console.log("got error", error);
  }
}

module.exports = { greet };
