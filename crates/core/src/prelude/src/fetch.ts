const STATUS_TEXT: Record<number, string> = {
  200: "OK",
  201: "Created",
  204: "No Content",
  301: "Moved Permanently",
  302: "Found",
  304: "Not Modified",
  400: "Bad Request",
  401: "Unauthorized",
  403: "Forbidden",
  404: "Not Found",
  405: "Method Not Allowed",
  408: "Request Timeout",
  409: "Conflict",
  429: "Too Many Requests",
  500: "Internal Server Error",
  502: "Bad Gateway",
  503: "Service Unavailable",
  504: "Gateway Timeout",
};

class _Headers {
  private _map: Record<string, string> = {};

  constructor(
    init?:
      | Record<string, string>
      | [string, string][]
      | _Headers,
  ) {
    if (!init) return;

    if (init instanceof _Headers) {
      Object.keys(init._map).forEach((key) => {
        this._map[key] = init._map[key];
      });
    } else if (Array.isArray(init)) {
      for (let i = 0; i < init.length; i++) {
        this._map[init[i][0].toLowerCase()] = init[i][1];
      }
    } else {
      Object.keys(init).forEach((key) => {
        this._map[key.toLowerCase()] = init[key];
      });
    }
  }

  append(name: string, value: string): void {
    const key = name.toLowerCase();
    if (key in this._map) {
      this._map[key] = this._map[key] + ", " + value;
    } else {
      this._map[key] = value;
    }
  }

  delete(name: string): void {
    delete this._map[name.toLowerCase()];
  }

  get(name: string): string | null {
    const val = this._map[name.toLowerCase()];
    return val !== undefined ? val : null;
  }

  has(name: string): boolean {
    return name.toLowerCase() in this._map;
  }

  set(name: string, value: string): void {
    this._map[name.toLowerCase()] = value;
  }

  forEach(
    callback: (value: string, key: string, parent: _Headers) => void,
  ): void {
    Object.keys(this._map).forEach((key) => {
      callback(this._map[key], key, this);
    });
  }

  entries(): [string, string][] {
    return Object.keys(this._map).map((k) => [k, this._map[k]] as [string, string]);
  }

  keys(): string[] {
    return Object.keys(this._map);
  }

  values(): string[] {
    return Object.keys(this._map).map((k) => this._map[k]);
  }
}

class _Response {
  readonly status: number;
  readonly statusText: string;
  readonly ok: boolean;
  readonly headers: _Headers;
  readonly url: string;
  private _body: string;
  private _bodyUsed: boolean = false;

  constructor(
    body: string | null,
    init?: { status?: number; statusText?: string; headers?: _Headers },
  ) {
    this._body = body || "";
    this.status = init?.status ?? 200;
    this.statusText = init?.statusText ?? STATUS_TEXT[this.status] ?? "";
    this.ok = this.status >= 200 && this.status < 300;
    this.headers = init?.headers ?? new _Headers();
    this.url = "";
  }

  get bodyUsed(): boolean {
    return this._bodyUsed;
  }

  text(): Promise<string> {
    if (this._bodyUsed) {
      return Promise.reject(new Error("Body has already been consumed"));
    }
    this._bodyUsed = true;
    return Promise.resolve(this._body);
  }

  json(): Promise<any> {
    if (this._bodyUsed) {
      return Promise.reject(new Error("Body has already been consumed"));
    }
    this._bodyUsed = true;
    return Promise.resolve(JSON.parse(this._body));
  }

  arrayBuffer(): Promise<ArrayBuffer> {
    if (this._bodyUsed) {
      return Promise.reject(new Error("Body has already been consumed"));
    }
    this._bodyUsed = true;
    const encoder = new TextEncoder();
    return Promise.resolve(encoder.encode(this._body).buffer);
  }

  clone(): _Response {
    if (this._bodyUsed) {
      throw new Error("Cannot clone a Response whose body has already been consumed");
    }
    const cloned = new _Response(this._body, {
      status: this.status,
      statusText: this.statusText,
      headers: new _Headers(this.headers),
    });
    return cloned;
  }
}

globalThis.fetch = function fetch(
  input: string | URL,
  init?: {
    method?: string;
    headers?: _Headers | Record<string, string> | [string, string][];
    body?: string;
  },
): Promise<_Response> {
  var url: string;
  if (typeof input === "string") {
    url = input;
  } else {
    url = input.toString();
  }

  var method = (init?.method ?? "GET").toUpperCase();
  var reqHeaders: Record<string, string | number | boolean> = {};

  if (init?.headers) {
    if (init.headers instanceof _Headers) {
      init.headers.forEach((value, key) => {
        reqHeaders[key] = value;
      });
    } else if (Array.isArray(init.headers)) {
      for (let i = 0; i < init.headers.length; i++) {
        reqHeaders[init.headers[i][0]] = init.headers[i][1];
      }
    } else {
      reqHeaders = init.headers as Record<string, string>;
    }
  }

  var body = init?.body;

  var httpReq: HttpRequest = {
    url: url,
    method: method as HttpRequest["method"],
    headers: reqHeaders,
  };

  try {
    var httpRes =
      body !== undefined ? Http.request(httpReq, body) : Http.request(httpReq);

    var responseHeaders = new _Headers(httpRes.headers || {});

    var response = new _Response(httpRes.body, {
      status: httpRes.status,
      statusText: STATUS_TEXT[httpRes.status] ?? "",
      headers: responseHeaders,
    });
    // Store the url on the response (readonly in spec, but we set it internally)
    (response as any).url = url;

    return Promise.resolve(response);
  } catch (err) {
    return Promise.reject(err);
  }
};

globalThis.Headers = _Headers;
globalThis.Response = _Response;

export {};
