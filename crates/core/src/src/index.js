import 'fast-text-encoding'
import 'core-js/actual/url';
import 'core-js/actual/url/to-json';
import 'core-js/actual/url-search-params';

import { URLPattern } from 'urlpattern-polyfill';

globalThis.URLPattern = URLPattern;

class __LazyPromise {
  constructor(executor) {
    if (typeof executor !== 'function') {
      throw new TypeError(`LazyPromise executor is not a function`);
    }
    this._executor = executor;
  }
  then() {
    this.promise = this.promise || new Promise(this._executor);
    return this.promise.then.apply(this.promise, arguments);
  }
}

globalThis.fopen = function (path) {
  return __fopen(path)
}

globalThis.read = function (fd, len) {
  const taskid = __read(fd, len.toString())
  console.log("kicked off read task id: " + taskid)
  return new __LazyPromise(resolve => {
      console.log("blocking __await on task id: " + taskid)
      let result = __await(taskid)
      console.log("resolving task id: " + taskid)
      //let result = '{\"bytes\": [1,2,3,4]}'
      resolve(JSON.parse(result))
  })
}
