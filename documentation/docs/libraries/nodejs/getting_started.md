---
description: Getting started with the official IOTA Wallet Library Software Node.js binding.
image: /img/logo/wallet_light.png
keywords:
- Node.js
- dotenv
- install
- npm
- yarn
- security
---
# Getting Started with Node.js

The [IOTA Wallet Node.js binding](https://www.npmjs.com/package/@iota/wallet) is published on [npmjs.com](https://www.npmjs.com/).

## Security

:::warning
In a production setup, do not store passwords in the host's environment variables or in the source code.  See our [backup and security recommendations](https://wiki.iota.org/introduction/guides/backup_security) for production setups.
:::

## Installation

The package is published in the [npmjs](https://www.npmjs.com/package/@iota/wallet). We also use _dotenv_ for password management in the examples.

- To install with NPM, you can run the following command:
```
$ npm install @iota/wallet dotenv
```
- To install with yarn, you can run the following command:
```
$ yarn install @iota/wallet dotenv
```

## Usage
You can find more information on using the `wallet.rs` library's node.js binding in the [how tos section](./how_to/0_create_account.mdx).