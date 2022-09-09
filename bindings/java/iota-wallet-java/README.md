# IOTA Wallet Library - Java binding

Java binding to the wallet.rs library.

To use the IOTA Java Wallet Library in your Java project, you must first build the library JAR for your operating
system.

## Build the JAR for your operating system (Linux, macOS, Windows)

**To build your JAR, you must ensure that you have the latest stable version of Rust installed.
Visit [Install Rust](https://www.rust-lang.org/tools/install) for installing Rust.
In addition, make sure you have the latest Java Development Kit (JDK) installed.**

1. Clone the repository: `git clone https://github.com/iotaledger/wallet.rs`
2. Change directory: `cd wallet.rs/bindings/java/iota-wallet-java`
3. If needed make `gradlew` executable: `chmod +x gradlew`
4. Build your JAR: `./gradlew jar`
5. Find the produced JAR in: `build/libs/`
6. Add the JAR as a library to your Java project.

After you linked the library, you can create a Wallet instance and interface with it.

```java
...
```

# Documentation

Please visit the [examples](../../../documentation/docs/libraries/java/getting_started.md) page for more information on using the IOTA Java Wallet Library.
More examples on how to use the library can be found [here](examples/ExampleProject/src).
In addition, since the IOTA Java library is similar to the IOTA Rust library, you might also want to look into Rust examples.