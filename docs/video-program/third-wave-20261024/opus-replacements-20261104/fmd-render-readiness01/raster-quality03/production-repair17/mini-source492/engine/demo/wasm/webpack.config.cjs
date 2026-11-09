const path = require("node:path");

module.exports = {
  mode: "production",
  target: ["web", "es2022"],
  entry: "./smoke.mjs",
  devtool: false,
  experiments: {
    asyncWebAssembly: true,
  },
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "smoke.js",
    clean: false,
  },
  performance: {
    hints: "error",
    maxAssetSize: 514521,
    maxEntrypointSize: 525000,
  },
  stats: "errors-warnings",
};
