const path = require('path');

module.exports = {
    plugins: [
        [
            '@docusaurus/plugin-content-docs',
            {
                id: 'cli-wallet',
                path: path.resolve(__dirname, 'docs'),
                routeBasePath: 'cli-wallet',
                sidebarPath: path.resolve(__dirname, 'sidebars.js'),
                editUrl: 'https://github.com/iotaledger/cli-wallet/edit/develop/documentation',
            }
        ],
    ],
    staticDirectories: [path.resolve(__dirname, 'static')],
};
