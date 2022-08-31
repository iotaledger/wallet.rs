# IOTA Wallet Library - Java binding

Java binding to the IOTA wallet library.

## Requirements

Ensure you have first installed the required dependencies for the library [here](https://github.com/iotaledger/wallet.rs/blob/dev/README.md).

## Installation

Clone project
```
git clone https://github.com/iotaledger/wallet.rs
```

Build the rust library
```
cd wallet.rs/bindings/java/native
cargo build
```

- Running an example using gradle
```
cd wallet.rs/bindings/java
./gradlew examples:basic-app:test --info
```

Make sure to make gradlew executable (`chmod +x gradlew`)

## Documentation

Documentation can be found [here](https://wallet-lib.docs.iota.org/libraries/nodejs).
