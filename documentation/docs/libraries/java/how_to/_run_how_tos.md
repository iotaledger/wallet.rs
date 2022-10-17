### Clone the Repository

To run the examples, you will first need to clone the repository. You can do so by running the following command:

```bash
git clone git@github.com:iotaledger/wallet.rs.git
```

Then, move into the Java code by running the following command:

```bash
cd wallet.rs/bindings/java/iota-wallet-java
```

## Run Code Examples

The IOTA Wallet Java library has numerous [examples](https://github.com/iotaledger/wallet.rs/tree/develop/bindings/java/iota-wallet-java/examples/src/main)
you can run to get acquainted with the library.  After you have followed the instructions to
[install the library](./../getting_started/java#install-the-library), you can run any example with the following
command from the `examples` directory:

```bash
./gradlew run -Pmain=CreateAccount
```

## Examples List

You can replace the `CreateAccount` by any other example from the [Java examples directory](https://github.com/iotaledger/wallet.rs/tree/develop/bindings/java/iota-wallet-java/examples/src/main).