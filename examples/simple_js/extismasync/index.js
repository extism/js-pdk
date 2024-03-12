const createPlugin = require('@extism/extism')

async function main() {
  const plugin = await createPlugin('../output.wasm')

  try {
  } finally {
    plugin.close()
  }
}

main().catch(err => (console.log(err), process.exit(1)))

