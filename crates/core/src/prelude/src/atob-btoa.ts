declare global {
  function atob(data: string): string;
  function btoa(data: string): string;
}

globalThis.btoa = function btoa(data: string): string {
  const str = String(data);
  const bytes = new Uint8Array(str.length);
  for (let i = 0; i < str.length; i++) {
    const code = str.charCodeAt(i);
    if (code > 255) {
      throw new DOMException(
        "The string to be encoded contains characters outside of the Latin1 range.",
        "InvalidCharacterError",
      );
    }
    bytes[i] = code;
  }
  return Host.arrayBufferToBase64(bytes.buffer);
};

globalThis.atob = function atob(data: string): string {
  const str = String(data).replace(/[\t\n\f\r ]/g, "");
  if (str.length % 4 === 1) {
    throw new DOMException(
      "The string to be decoded is not correctly encoded.",
      "InvalidCharacterError",
    );
  }
  const buffer = Host.base64ToArrayBuffer(str);
  const bytes = new Uint8Array(buffer);
  let result = "";
  for (let i = 0; i < bytes.length; i++) {
    result += String.fromCharCode(bytes[i]);
  }
  return result;
};

export {};
