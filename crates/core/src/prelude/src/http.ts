declare global {
  interface HttpRequest {
    url: string;
    method?:
      | "GET"
      | "HEAD"
      | "POST"
      | "PUT"
      | "DELETE"
      | "CONNECT"
      | "OPTIONS"
      | "TRACE"
      | "PATCH";
    headers?: Record<string, string | number | boolean>;
  }

  interface HttpResponse {
    body: string;
    status: number;
  }

  var Http: {
    request(req: HttpRequest, body?: string | ArrayBufferLike): HttpResponse;
  };
}

Http.request = new Proxy(Http.request, {
  apply(target, thisArg, [req, body]) {
    // convert body to string if it's an arraybuffer
    if (typeof body === "object" && "byteLength" in body) {
      body = new Uint8Array(body).toString();
    }

    return Reflect.apply(
      target,
      thisArg,
      // TODO: We need to completely avoid passing a second argument due to a bug in the runtime,
      // which converts `undefined` to `"undefined"`. This is also the case for req.method.
      body !== undefined ? [req, body] : [req]
    );
  },
});

export {};
