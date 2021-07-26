# IOTA Wallet Library - Node.js binding

Node.js binding to the IOTA wallet library.

## Requirements (only for building the binary yourself)

Ensure you have first installed the required dependencies for the library [here](https://github.com/iotaledger/wallet.rs/blob/dev/README.md) and on Windows also LLVM, our workflow uses `https://github.com/llvm/llvm-project/releases/download/llvmorg-11.0.1/LLVM-11.0.1-win64.exe`. You might also need to set an environment variable `RUSTFLAGS` to `-C target-feature=+crt-static`.

## Installation

- Using NPM:
```
npm i @iota/wallet
```
- Using yarn: 
```
yarn add @iota/wallet
```

## Documentation

Documentation can be found [here](https://wallet-lib.docs.iota.org/libraries/nodejs).
