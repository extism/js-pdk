const esbuild = require("esbuild");
const path = require("path");

esbuild.build({
  entryPoints: ["src/index.js"],
  outdir: "dist",
  bundle: true,
  format: "cjs",
  target: ["es2020"],
  plugins: [
    {
      name: "buffer-shim",
      setup(build) {
        // Redirect require('buffer') to our shim that uses the global Buffer
        // provided by the Extism JS PDK runtime
        build.onResolve({ filter: /^buffer$/ }, () => ({
          path: path.resolve(__dirname, "buffer-shim.js"),
        }));
      },
    },
  ],
});
