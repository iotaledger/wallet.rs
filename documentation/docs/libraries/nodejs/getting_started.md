# Getting Started with Node.js

The [IOTA Wallet Node.js binding](https://www.npmjs.com/package/@iota/wallet) is published on [npmjs.com](https://www.npmjs.com/).

:::info
You can find a guide for exchanges and the most common use cases in the [Chrysalis documentation](https://chrysalis.docs.iota.org/guides/exchange_guide.html), which is based on `wallet.rs` and `Node.js`. 
:::

## Security

:::warning
In a production setup, do not store passwords in the host's environment variables or in the source code.  See our [backup and security recommendations](https://chrysalis.docs.iota.org/guides/backup_security.html) for production setups.
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
You can find more information on using the `wallet.rs` library's node.js binding in the [examples section](examples.md).