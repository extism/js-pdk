import 'fast-text-encoding'
import 'core-js/actual/url';
import 'core-js/actual/url/to-json';
import 'core-js/actual/url-search-params';

import { URLPattern } from 'urlpattern-polyfill';

globalThis.URLPattern = URLPattern;

