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
- getting started
---
# Getting Started with Node.js

The [IOTA Wallet Node.js binding](https://www.npmjs.com/package/@iota/wallet) is published on [npmjs.com](https://www.npmjs.com/).

:::note

You can find a guide for exchanges and the most common use cases in the [Chrysalis documentation](https://wiki.iota.org/chrysalis-docs/guides/exchange), which is based on `wallet.rs` and `Node.js`. 

:::

## Security

:::note

In a production setup, do not store passwords in the host's environment variables or in the source code. For reference, see our [backup and security recommendations](https://chrysalis.docs.iota.org/guides/backup_security) for production setups.

:::

## Installation

The package is published in the [npmjs](https://www.npmjs.com/package/@iota/wallet). We also use _dotenv_ for password management in the examples.

- To install with NPM, you can run the following command:
```
npm install @iota/wallet@1 dotenv
```
- To install with yarn, you can run the following command:
```
yarn add @iota/wallet@1 dotenv
```

## Usage

You can find more information on using the `wallet.rs` library's node.js binding in the [examples section](../examples/nodejs.mdx).