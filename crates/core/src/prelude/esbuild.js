const esbuild = require('esbuild');

esbuild
  .build({
    entryPoints: ['src/index.js'],
    outdir: 'dist',
    bundle: true,
    sourcemap: true,
    minify: false,
    format: 'cjs', // needs to be CJS for now
    target: ['es2020'] // don't go over es2020 because quickjs doesn't support it
  })
