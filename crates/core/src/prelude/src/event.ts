class _Event {
  readonly type: string;
  readonly bubbles: boolean;
  readonly cancelable: boolean;
  readonly timeStamp: number;
  target: any = null;
  currentTarget: any = null;
  private _defaultPrevented: boolean = false;
  private _stopPropagationFlag: boolean = false;
  private _stopImmediatePropagationFlag: boolean = false;

  constructor(
    type: string,
    eventInitDict?: { bubbles?: boolean; cancelable?: boolean },
  ) {
    this.type = type;
    this.bubbles = eventInitDict?.bubbles ?? false;
    this.cancelable = eventInitDict?.cancelable ?? false;
    this.timeStamp = performance.now();
  }

  get defaultPrevented(): boolean {
    return this._defaultPrevented;
  }

  preventDefault(): void {
    if (this.cancelable) this._defaultPrevented = true;
  }

  stopPropagation(): void {
    this._stopPropagationFlag = true;
  }

  stopImmediatePropagation(): void {
    this._stopPropagationFlag = true;
    this._stopImmediatePropagationFlag = true;
  }

  get _immediateStopped(): boolean {
    return this._stopImmediatePropagationFlag;
  }
}

interface ListenerEntry {
  callback: Function;
  once: boolean;
}

class _EventTarget {
  private _listeners: Record<string, ListenerEntry[]> = {};

  addEventListener(
    type: string,
    callback: Function | null,
    options?: { once?: boolean } | boolean,
  ): void {
    if (!callback) return;

    const once =
      typeof options === "object" ? (options.once ?? false) : false;

    if (!this._listeners[type]) {
      this._listeners[type] = [];
    }

    // Don't add duplicates (same callback)
    for (let i = 0; i < this._listeners[type].length; i++) {
      if (this._listeners[type][i].callback === callback) return;
    }

    this._listeners[type].push({ callback, once });
  }

  removeEventListener(type: string, callback: Function | null): void {
    if (!callback || !this._listeners[type]) return;
    this._listeners[type] = this._listeners[type].filter(
      (l) => l.callback !== callback,
    );
  }

  dispatchEvent(event: _Event): boolean {
    const listeners = this._listeners[event.type];
    if (!listeners || listeners.length === 0) return !event.defaultPrevented;

    event.target = this;
    event.currentTarget = this;

    const snapshot = listeners.slice();
    const toRemove: Function[] = [];

    for (let i = 0; i < snapshot.length; i++) {
      snapshot[i].callback.call(this, event);
      if (snapshot[i].once) toRemove.push(snapshot[i].callback);
      if (event._immediateStopped) break;
    }

    for (let i = 0; i < toRemove.length; i++) {
      this.removeEventListener(event.type, toRemove[i]);
    }

    return !event.defaultPrevented;
  }
}

globalThis.Event = _Event as any;
globalThis.EventTarget = _EventTarget as any;

export {};
