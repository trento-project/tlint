const path = require("path");
const CopyPlugin = require("copy-webpack-plugin");
const { CleanWebpackPlugin } = require("clean-webpack-plugin");
const dist = path.resolve(__dirname, "dist");

module.exports = {
  mode: process.env.NODE_ENV || "development",
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
    new CleanWebpackPlugin(),
  ],
  experiments: {
    asyncWebAssembly: true,
  },
  optimization: {
    minimize: true,
    splitChunks: {
        chunks: 'all',
    },
  },
};
