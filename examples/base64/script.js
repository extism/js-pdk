function greet() {
  const base64String = "SGVsbG8g8J+MjSBXb3JsZCHwn4yN";

  console.log('decoding string:', base64String);

  const decodedBuffer = Host.base64ToArrayBuffer(base64String);
  const decodedString = new TextDecoder().decode(decodedBuffer);

  console.log('decoded string:', decodedString);

  const encodedBuffer = Host.arrayBufferToBase64(decodedBuffer);
  
  console.log('encoded string:', encodedBuffer);

  Host.outputString(`Hello, ${Host.inputString()}`)
}

module.exports = { greet };
