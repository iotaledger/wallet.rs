# Getting Started with Node.js

The [IOTA Wallet Node.js binding](https://www.npmjs.com/package/@iota/wallet) is published on npmjs.org.

> There is a guide for exchanges [available](https://chrysalis.docs.iota.org/guides/exchange_guide.html) which is based on `wallet.rs` and `Node.js`. It also covers several most common use cases.

## Security
Please note: In is not recommended to store passwords on host's environment variables or in the source code in a production setup! Please make sure you follow our [backup and security](https://chrysalis.docs.iota.org/guides/backup_security.html) recommendations for production use!

## Installation

Currently the package isn't published so you'd need to link it to your project using `npm` or `yarn`. We also use `dotenv` for password management in the examples.

- Using NPM:
```
$ npm install @iota/wallet dotenv
```
- Using yarn: 
```
$ yarn install @iota/wallet dotenv
```

## Usage

```javascript
{{ #include ../../../bindings/nodejs/examples/1-create-account.js }}
```