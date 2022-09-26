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
import org.iota.Wallet;
import org.iota.types.AccountHandle;
import org.iota.types.ClientConfig;
import org.iota.types.WalletConfig;
import org.iota.types.exceptions.WalletException;
import org.iota.types.secret.MnemonicSecretManager;

public class CreateAccounts {
    private static final String SHIMMER_TESTNET_NODE_URL = "https://api.testnet.shimmer.network";
    private static final int SHIMMER_COIN_TYPE = 4219;
    private static final String DEFAULT_DEVELOPMENT_MNEMONIC = "hidden enroll proud copper decide negative orient asset speed work dolphin atom unhappy game cannon scheme glow kid ring core name still twist actor";

    public static void main(String[] args) throws WalletException {
        // Build the wallet.
        Wallet wallet = new Wallet(new WalletConfig()
                .withClientOptions(new ClientConfig().withNodes(new String[] { SHIMMER_TESTNET_NODE_URL }))
                .withSecretManager(new MnemonicSecretManager(DEFAULT_DEVELOPMENT_MNEMONIC))
                .withCoinType(SHIMMER_COIN_TYPE)
        );

        // Create an account.
        AccountHandle a = wallet.createAccount("Hans");

        // Print the account.
        System.out.println(a);
    }
}
```

# Documentation

Please visit the [examples](../../../documentation/docs/libraries/java/getting_started.md) page for more information on using the IOTA Java Wallet Library.
More examples on how to use the library can be found [here](examples/ExampleProject/src).
In addition, since the IOTA Java library is similar to the IOTA Rust library, you might also want to look into Rust examples.