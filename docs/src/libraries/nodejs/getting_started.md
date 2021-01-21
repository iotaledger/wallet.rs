# Getting Started with Node.js

## Requirements

Ensure you have first installed the required dependencies for the library [here](https://github.com/iotaledger/wallet.rs/blob/develop/README.md).

## Installation

Currently the package isn't published so you'd need to link it to your project using `npm` or `yarn`.

- Using NPM:
```
$ git clone https://github.com/iotaledger/wallet.rs
$ cd wallet.rs/bindings/node
$ npm link
$ cd /path/to/nodejs/project/
$ npm link iota-wallet
```
- Using yarn: 
```
$ git clone https://github.com/iotaledger/wallet.rs
$ cd wallet.rs/bindings/node
$ yarn link
$ cd /path/to/nodejs/project/
$ yarn link iota-wallet
```

## Setup your project

npm install iota-wallet

## Example 

```javascript
const { AccountManager, StorageType, SignerType } = require('iota-wallet')
const manager = new AccountManager({
    storagePath: './storage',
    storageType: StorageType.Sqlite
})
const account = await manager.createAccount({
  alias: 'Account1',
  clientOptions: { node: 'http://api.lb-0.testnet.chrysalis2.com', localPow: false },
  signerType: SignerType.EnvMnemonic
})
account.sync()
```