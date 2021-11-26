# IOTA Wallet Library - Java binding

Java binding to the IOTA wallet library.

## Requirements

Ensure you have first installed the required dependencies for the library [here](https://github.com/iotaledger/wallet.rs/blob/dev/README.md).

## Installation

Clone project
```
git clone https://github.com/iotaledger/wallet.rs
```

Build the rust library (This generates the java code)
```
cd wallet.rs/bindings/java/native
cargo build --release
```

Make gradlew executable (`chmod +x gradlew`)


Running the java example using gradle
```
./gradlew examples:java-app:test --info
```

Running the android app using gradle:

Specific instructions in `wallet.rs/bindings/java/examples/android-app/README.md`

## Documentation

Documentation can be found [here](https://wallet-lib.docs.iota.org/libraries/nodejs).
