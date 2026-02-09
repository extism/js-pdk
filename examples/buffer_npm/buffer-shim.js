// Shim for require('buffer') that uses the PDK-provided global Buffer
module.exports.Buffer = globalThis.Buffer;
