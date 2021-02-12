# Examples

There are several examples to show the usage of the library.

> All examples can be found in [/bindings/nodejs/examples](https://github.com/iotaledger/wallet.rs/bindings/nodejs/examples/)

## Setup
First, setup your environment as follows:

```
git clone https://github.com/iotaledger/wallet.rs
cd bindings/node/examples
npm install
cp .env.example .env
```

Add your custom password to the `.env` file.

## 1. Example: Create an Account

Run:
```
node 1-create-account.js
```

Code:
```javascript
{{ #include ../../../bindings/nodejs/examples/1-create-account.js }}
```

## 2. Generate Address
Run:
```
node 2-generate-address.js
```

Code:
```javascript
{{ #include ../../../bindings/nodejs/examples/2-generate-address.js }}
```

## 3. Example: Check Balance
Run:
```
node 3-check_balance
```

Code:
```javascript
{{ #include ../../../bindings/nodejs/examples/3-check_balance.js }}
```

## 4. Example: Check Balance
Now you can send the test tokens to an address! 

Run
```
node 4-send.js
```

Code:
```javascript
{{ #include ../../../bindings/nodejs/examples/4-send.js }}
```

## 5. Backup

Run
```
node 5-backup.js
```

Code:
```javascript
{{ #include ../../../bindings/nodejs/examples/5-backup.js }}
```

## 6. Restore

Run
```
node 6-restore.js
```

Code:
```javascript
{{ #include ../../../bindings/nodejs/examples/6-restore.js }}
```