# IOTA Wallet Library - Java binding

Java binding to the IOTA wallet library.

## Requirements

Ensure you have first installed the required dependencies for the library [here](https://github.com/iotaledger/wallet.rs/blob/dev/README.md).

## Installation

Clone project
```
git clone https://github.com/iotaledger/wallet.rs
```

Build the rust library (This generates the java source code and JNI library file)
```
cd wallet.rs/bindings/java
cargo build --release
```

Source code will be generated under `wallet.rs/bindings/java/native/src/main/java/org/iota/wallet`

Binaries can be found at `wallet.rs/bindings/java/target/release`

Once this step succeeds we need to generate the jar file containing the newly generated Java source files.
### Gradle

Make `gradlew` executable (`chmod +x gradlew`) if needed, then run
```
cd wallet.rs/bindings/java
./gradlew jar
```

### Maven
```
cd wallet.rs/bindings/java
mvn install
```

The jar will be found at `wallet.rs/bindings/java/native/build/libs/native.jar`

## Running the Java example

### Gradle
```
./gradlew examples:java-app:test --info
```

### Maven
```
mvn exec:exec
```

## Running the Java example
The Android app needs further compilation instructions.

Specific instructions in `wallet.rs/bindings/java/examples/android-app/README.md`

## Documentation

Documentation can be found [here](https://wallet-lib.docs.iota.org/docs/specification).
