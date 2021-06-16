# Getting Started with Node.js

The [IOTA Wallet Node.js binding](https://www.npmjs.com/package/@iota/wallet) is published on [npmjs.com](https://www.npmjs.com/).

:::info
You can find a guide for exchanges and the most common use cases in the [Chrysalis documentation](https://chrysalis.docs.iota.org/guides/exchange_guide.html), which is based on `Wallet.rs` and `Node.js`. 
:::

## Security
:::warning
It is not recommended to store passwords on the host's environment variables, or in the source code in a production setup. 
Please make sure you follow our [backup and security recommendations](https://chrysalis.docs.iota.org/guides/backup_security.html) for production use.
:::
## Installation

Currently, the package isn't published,  so you'll need to link it to your project using `npm` or `yarn`. We also use `dotenv` for password management in the examples.

- To install with NPM, you can run the following command:
```
$ npm install @iota/wallet dotenv
```
- To install with yarn, you can run the following command:
```
$ yarn install @iota/wallet dotenv
```

## Usage
You can find more information on using the `Wallet.rs` library's node.js binding in the [examples section](examples.md).