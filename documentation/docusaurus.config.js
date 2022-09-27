const path = require('path');

module.exports = {
  plugins: [
    [
      '@docusaurus/plugin-content-docs',
      {
        id: 'wallet-rs',
        path: path.resolve(__dirname, 'docs'),
        routeBasePath: 'wallet.rs',
        sidebarPath: path.resolve(__dirname, 'sidebars.js'),
        editUrl: 'https://github.com/iotaledger/wallet.rs/edit/develop/documentation',
        remarkPlugins: [require('remark-code-import'), require('remark-import-partial')],
      }
    ],
  ],
  staticDirectories: [path.resolve(__dirname, 'static')],
};
