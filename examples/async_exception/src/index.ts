import { createHighlighter } from 'shiki'

// more context:
// - https://github.com/DelSkayn/rquickjs/issues/421
// - https://github.com/DelSkayn/rquickjs/pull/422
// - https://github.com/quickjs-ng/quickjs/issues/39
// - https://github.com/quickjs-ng/quickjs/pull/1038

export function main() {
  try {
    const code = 'const a = 1' // input code
    const highlighter = createHighlighter({
      themes: ['nord'],
      langs: ['javascript'],
    })

    console.log('BAAM!')

    //@ts-ignore - `codeToHtml` doesn't exist, `highlighter` is not awaited, so it's still a Promise
    const html = highlighter.codeToHtml(code, {
      lang: 'javascript',
      theme: 'nord'
    })

    Host.outputString(html);
  } catch (error) {
    console.log('error: ', error)
    throw error
  }
}

// extism call ../issue_134.wasm main --wasi --log-level info