const CopyWebpackPlugin = require("copy-webpack-plugin");
const path = require('path');

module.exports = {
  entry: "./bootstrap.js",
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "bootstrap.js",
  },
  mode: "development",
  plugins: [
    new CopyWebpackPlugin(['index.html'])
  ],
  devServer: {
    proxy: {
      '/api': {
        target: 'https://icfpc2020-api.testkontur.ru',
        pathRewrite: { '^/api': '' },
        secure: false,
        changeOrigin: true,
      }
    }
  }
};
