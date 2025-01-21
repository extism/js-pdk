/**
 * An example of a plugin that uses Console extensively, CJS flavored plug-in:
 */

function greet() {
  const n = 1;
  tryPrint('n + 1', n + 1);
  tryPrint('multiple string args', 'one', 'two', 'three');
  tryPrint('single n', n);
  tryPrint('three ns', n, n, n);
  tryPrint('n with label', 'n', n);
  tryPrint('boolean', true);
  tryPrint('null', null);
  tryPrint('undefined', undefined);
  tryPrint('empty object', {});
  tryPrint('empty array', []);
  tryPrint('object with key', { key: 'value' });
  console.warn('This is a warning', 123);
  console.error('This is an error', 456);
  console.info('This is an info', 789);
  console.debug('This is a debug', 101112);
  console.trace('This is a trace', 131415);

  console.log('This is an object', { key: 'value' });
  console.log('This is an array', [1, 2, 3]);
  console.log('This is a string', 'Hello, World!');
  console.log('This is a number', 123);
  console.log('This is a boolean', true);
  console.log('This is a null', null);
  console.log('This is an undefined', undefined);
  console.log('This is a function', function() {});
  console.log('This is a symbol', Symbol('test'));
  console.log('This is a date', new Date());
  console.log('This is an error', new Error('Hi there!'));
  console.log('This is a map', new Map([[1, 'one'], [2, 'two']] ));
}

function tryPrint(text, ...args) {
  try {
    console.log(...args);
    console.log(`${text} - ✅`);
  } catch (e) {
    console.log(`${text} - ❌ - ${e.message}`);
  }
  console.log('------')
}


module.exports = { greet };
