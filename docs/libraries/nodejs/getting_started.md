# Getting Started with Node.js

The [IOTA Wallet Node.js binding](https://www.npmjs.com/package/@iota/wallet) is published on npmjs.org.

> There is a guide for exchanges [available](https://chrysalis.docs.iota.org/guides/exchange_guide.html) which is based on `wallet.rs` and `Node.js`. It also covers several most common use cases.

## Security
Please note: In is not recommended to store passwords on host's environment variables or in the source code in a production setup! Please make sure you follow our [backup and security](https://chrysalis.docs.iota.org/guides/backup_security.html) recommendations for production use!

## Requirements (only for building the binary yourself)

Ensure you have first installed the required dependencies for the library [here](https://github.com/iotaledger/wallet.rs/blob/dev/README.md) and on Windows also LLVM, our workflow uses `https://github.com/llvm/llvm-project/releases/download/llvmorg-11.0.1/LLVM-11.0.1-win64.exe`.

## Installation

- Using NPM:
```
npm i @iota/wallet
```
- Using yarn: 
```
yarn add @iota/wallet
```

## Usage

```javascript
{{ #include ../../../bindings/nodejs/examples/1-create-account.js }}
```