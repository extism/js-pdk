export function greet() {
  let extra = new TextEncoder().encode("aaa")
  let decoded = new TextDecoder().decode(extra)
  Host.outputString(`Hello, ${Host.inputString()} ${decoded}`)
}
