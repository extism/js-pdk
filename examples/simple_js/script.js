/**
 * A simple example of a JavaScript, CJS flavored plug-in:
 */


function callHttp() {
  const request = {
    method: "GET",
    url: "https://jsonplaceholder.typicode.com/todos/1"
  }
  const response = Http.request(request)
  if (response.status != 200) throw new Error(`Got non 200 response ${response.status}`)
  Host.outputString(response.body)
}

module.exports = { callHttp }
