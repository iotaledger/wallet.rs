# Examples

There are several examples to show the usage of the library.

## Setup
First, setup the example environment.

```
cd bindings/node/examples
npm install
copy .env.example .env
```

Add your mnemonic to the `.env` file.

Make sure you generate a 24-word BIP39 mnemonic in your .env file and define a node; see
`.env.example` for a boilerplate. To generate a mnemonic you can utilize the BIP39 standard
for that; We encourage you to generate this offline with trusted tools but if you need a 
quick mnemonic for testing only you can use something like https://github.com/iancoleman/bip39

## 1. Example: Create an Account

This example creates a new database and account. Please make sure you have the .env variables configured

Run the example:

```
node 1-create-account.js
```

## 2. Example: Create Syncronize
Get some test tokens from the [IOTA faucet](https://faucet.testnet.chrysalis2.com/) and sync your account.

```
node 2-sync-and-check.js
```

## 2. Example: Send IOTA Tokens.
Now you can send the test tokens to an address! 
```
node 3-send.js
```