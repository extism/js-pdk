async function greet() {
  var pass = true;

  // --- Test Headers class ---
  var h = new Headers();
  h.set("Content-Type", "application/json");
  h.set("X-Custom", "test");

  // case-insensitive get
  if (h.get("content-type") !== "application/json") {
    console.error("FAIL: Headers.get case-insensitive");
    pass = false;
  }

  if (!h.has("x-custom")) {
    console.error("FAIL: Headers.has case-insensitive");
    pass = false;
  }

  // append
  h.append("X-Custom", "test2");
  if (h.get("x-custom") !== "test, test2") {
    console.error("FAIL: Headers.append");
    pass = false;
  }

  // delete
  h.delete("x-custom");
  if (h.has("x-custom")) {
    console.error("FAIL: Headers.delete");
    pass = false;
  }

  // constructor with object
  var h2 = new Headers({ "Accept": "text/html", "X-Foo": "bar" });
  if (h2.get("accept") !== "text/html" || h2.get("x-foo") !== "bar") {
    console.error("FAIL: Headers constructor with object");
    pass = false;
  }

  // constructor with array of tuples
  var h3 = new Headers([["Content-Type", "text/plain"], ["Accept", "application/json"]]);
  if (h3.get("content-type") !== "text/plain") {
    console.error("FAIL: Headers constructor with array");
    pass = false;
  }

  // forEach
  var count = 0;
  h3.forEach(function () { count++; });
  if (count !== 2) {
    console.error("FAIL: Headers.forEach count, got:", count);
    pass = false;
  }

  // entries, keys, values
  if (h3.keys().length !== 2) {
    console.error("FAIL: Headers.keys()");
    pass = false;
  }

  console.log("Headers tests: OK");

  // --- Test Response class ---
  var res = new Response('{"message":"hello"}', {
    status: 200,
    statusText: "OK",
    headers: new Headers({ "Content-Type": "application/json" }),
  });

  if (res.status !== 200) {
    console.error("FAIL: Response.status");
    pass = false;
  }
  if (res.statusText !== "OK") {
    console.error("FAIL: Response.statusText");
    pass = false;
  }
  if (!res.ok) {
    console.error("FAIL: Response.ok");
    pass = false;
  }
  if (res.headers.get("content-type") !== "application/json") {
    console.error("FAIL: Response.headers");
    pass = false;
  }

  // clone before consuming body
  var cloned = res.clone();

  // text()
  var bodyText = await res.text();
  if (bodyText !== '{"message":"hello"}') {
    console.error("FAIL: Response.text()");
    pass = false;
  }

  // bodyUsed after consuming
  if (!res.bodyUsed) {
    console.error("FAIL: Response.bodyUsed after text()");
    pass = false;
  }

  // json() on cloned response
  var obj = await cloned.json();
  if (obj.message !== "hello") {
    console.error("FAIL: Response.json()");
    pass = false;
  }

  // 404 response should not be ok
  var res404 = new Response("Not Found", { status: 404 });
  if (res404.ok) {
    console.error("FAIL: 404 Response.ok should be false");
    pass = false;
  }
  if (res404.status !== 404) {
    console.error("FAIL: 404 Response.status");
    pass = false;
  }

  console.log("Response tests: OK");

  // --- Test fetch() with real HTTP (Promise-based .then) ---
  var thenContent = "";
  var thenPass = false;
  fetch("http://example.com").then(function (r) {
    if (r.status !== 200) {
      console.error("FAIL: fetch then status, got:", r.status);
      return;
    }
    if (!r.ok) {
      console.error("FAIL: fetch then ok");
      return;
    }
    if (r.url !== "http://example.com") {
      console.error("FAIL: fetch then url, got:", r.url);
      return;
    }
    r.text().then(function (body) {
      thenContent = body;
      if (body.indexOf("Example Domain") === -1) {
        console.error("FAIL: fetch then body missing expected content");
        return;
      }
      thenPass = true;
      console.log("fetch GET (then): OK");
    });
  });

  // yield to let the .then microtasks execute
  await Promise.resolve();
  await Promise.resolve();

  if (!thenPass) {
    console.error("FAIL: fetch .then chain did not complete");
    pass = false;
  }

  // --- Test fetch() with await in async function ---
  var asyncResult = await asyncFetchTest();
  if (!asyncResult.ok) {
    pass = false;
  }

  if (!pass) {
    throw new Error("fetch tests failed\n| then-content = " + thenContent + "\n| async-content = " + asyncResult.asyncContent);
  }

  Host.outputString("fetch: all tests passed" + "\n| then-content = " + thenContent + "\n| async-content = " + asyncResult.asyncContent);
}

async function asyncFetchTest() {
  var r = await fetch("http://example.com");
  if (r.status !== 200) {
    throw new Error("FAIL: await fetch status, got: " + r.status);
  }
  if (!r.ok) {
    throw new Error("FAIL: await fetch ok");
  }
  var body = await r.text();
  if (body.indexOf("Example Domain") === -1) {
    throw new Error("FAIL: await fetch body missing expected content");
  }
  console.log("fetch GET (await): OK");
  return { ok: true, asyncContent: body }
}

module.exports = { greet };
