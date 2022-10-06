# IOTA Wallet Library - Java binding

Java binding to the wallet.rs library.

To use the IOTA Java Wallet Library in your Java project, you must first build the library JAR for your operating
system.

## Requirements

* The latest [Java Development Kit (JDK)](https://www.oracle.com/java/technologies/downloads/).
* [Rust and Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) to compile the binding.
* (for Linux only) `libudev`. You can install it with `apt install libudev-dev`.

## Build the JAR for Your Operating System (Linux, macOS, Windows)

To use the IOTA Java Wallet Library in your Java project, you must first build the library JAR for your operating
system.

### Clone the Repository

You can clone the [wallet.rs wallet library](https://github.com/iotaledger/wallet.rs) by running the following command:

```bash
git clone git@github.com:iotaledger/wallet.rs.git
```

### Change to the Java Binding Directory

After you have cloned the repository, you should change directory to `wallet.rs/bindings/java/iota-wallet-java`. You can do so by
running the following command:

```bash
cd wallet.rs/bindings/java/iota-wallet-java
```

### Make `gradlew` Executable

If needed, you can make the `gradlew` file executable by running the following command:

```bash
chmod +x gradlew
```

### Build Your JAR

You can now build your JAR file by running the following command:

```bash
./gradlew jar
```

This will produce a `JAR` file in `build/libs/` which you can add to your Java project.

## Use the Library

After you linked the library, you can create a Wallet instance and interface with it as shown in the following snippet:

```java
import org.iota.Wallet;
import org.iota.types.AccountHandle;
import org.iota.types.ClientConfig;
import org.iota.types.CoinType;
import org.iota.types.WalletConfig;
import org.iota.types.exceptions.WalletException;
import org.iota.types.secret.StrongholdSecretManager;

public class CreateAccount {
    private static final String DEFAULT_DEVELOPMENT_MNEMONIC = "hidden enroll proud copper decide negative orient asset speed work dolphin atom unhappy game cannon scheme glow kid ring core name still twist actor";

    public static void main(String[] args) throws WalletException {
        // Build the wallet.
        Wallet wallet = new Wallet(new WalletConfig()
                .withClientOptions(new ClientConfig().withNodes("https://api.testnet.shimmer.network"))
                .withSecretManager(new StrongholdSecretManager("PASSWORD_FOR_ENCRYPTION", null, "example-wallet"))
                .withCoinType(CoinType.Shimmer)
        );
        wallet.storeMnemonic(DEFAULT_DEVELOPMENT_MNEMONIC);

        // Create an account.
        AccountHandle a = wallet.createAccount("Hans");

        // Print the account.
        System.out.println(a);
    }
}
```

# Documentation

Please visit the [Shimmer Wiki](https://wiki.shimmer.network) for more information on using the IOTA Java Wallet Library.
More examples on how to use the library can be found [here](examples/ExampleProject/src).