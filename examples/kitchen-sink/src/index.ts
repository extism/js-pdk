export function greet() {
  let input = Host.inputString();
  Var.set("input", input);
  const req: HttpRequest = {
    url: "https://postman-echo.com/post",
    method: "POST",
  };
  let varInput = Var.getString("input");
  if (!varInput) {
    Host.outputString("failed to get var: input");
    return -1;
  }
  let resp = Http.request(req, varInput);
  if (resp.status !== 200) {
    return -2;
  }
  const body = JSON.parse(resp.body);
  if (body.data !== input) {
    Host.outputString("got unexpected output: " + body.data);
    return -3;
  }

  const configLastName = Config.get("last_name");
  if (!configLastName) {
    Host.outputString("failed to get config: last_name");
    return -4;
  }

  if (`${body.data} ${configLastName}` !== "Steve Manuel") {
    Host.outputString(`got unexpected output: ${body.data} ${configLastName}`);
    return -5;
  }

  const mem = Memory.fromString("Hello, " + body.data + " " + configLastName);
  Host.outputString(mem.readString()); // TODO: ideally have a way to output memory directly
  mem.free();
  return 0;
}
