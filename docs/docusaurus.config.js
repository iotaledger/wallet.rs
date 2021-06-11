/** @type {import('@docusaurus/types').DocusaurusConfig} */
module.exports = {
  title: 'IOTA Wallet Library',
  tagline: '',
  url: 'https://wallet-lib.docs.iota.org/',
  baseUrl: '/',
  onBrokenLinks: 'warn',
  onBrokenMarkdownLinks: 'throw',
  favicon: 'img/favicon.ico',
  organizationName: 'iotaledger', // Usually your GitHub org/user name.
  projectName: 'wallet.rs', // Usually your repo name.
  themeConfig: {
    navbar: {
      title: 'Wallet.rs documentation',
      logo: {
        alt: 'IOTA',
        src: 'static/img/logo/Logo_Swirl_Dark.png',
      },
    },
  },
  presets: [
    [
      '@docusaurus/preset-classic',
      {
        docs: {
          sidebarPath: require.resolve('./sidebars.js'),
          routeBasePath:'/',
          // Please change this to your repo.
          editUrl:
            'https://github.com/iotaledger/wallet.rs/tree/develop/docs',
        },
        theme: {
          customCss: require.resolve('./src/css/custom.css'),
        },
      },
    ],
  ],
};
