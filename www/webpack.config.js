const path = require("path");
const CopyPlugin = require("copy-webpack-plugin");
const dist = path.resolve(__dirname, "dist");

module.exports = {
  // mode: "production",
  mode: "development",
  entry: "./js/bootstrap.js",
  output: {
    path: dist,
    filename: "main.js",
    globalObject: "this",
  },
  devServer: {
    static: dist,
  },
  plugins: [
    new CopyPlugin([
      path.resolve(__dirname, "static")
    ]),
  ],
  experiments: {
    asyncWebAssembly: true,
  },
};
