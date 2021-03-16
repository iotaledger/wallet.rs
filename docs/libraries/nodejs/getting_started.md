# Getting Started with Node.js

The [IOTA Wallet Node.js binding](https://www.npmjs.com/package/@iota/wallet) is published on npmjs.org.

> There is a guide for exchanges [available](https://chrysalis.docs.iota.org/guides/exchange_guide.html) which is based on `wallet.rs` and `Node.js`. It also covers several most common use cases.

## Installation

Currently the package isn't published so you'd need to link it to your project using `npm` or `yarn`. We also recommend to use `dotenv` for password management.

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