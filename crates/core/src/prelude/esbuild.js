const esbuild = require("esbuild");

esbuild
  .build({
    entryPoints: ["src/index.ts"],
    outdir: "dist",
    bundle: true,
    minify: true,
    format: "iife", 
    target: ["es2020"], // don't go over es2020 because quickjs doesn't support it
  });
