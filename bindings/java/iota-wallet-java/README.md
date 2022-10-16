# IOTA Wallet - Java binding library

Get started with the official IOTA Wallet Java binding library.

## Requirements

* Make sure you have the latest [Java Development Kit (JDK)](https://www.oracle.com/java/technologies/downloads/) installed.

## Download

Gradle:
```gradle
dependencies {
  implementation 'org.iota:iota-wallet-java:1.0.0-rc.1'
}
```

Maven:
```xml
<dependency>
  <groupId>org.iota</groupId>
  <artifactId>iota-wallet-java</artifactId>
  <version>1.0.0-rc.1</version>
</dependency>
```

## Example

```java
// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

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
        AccountHandle a = wallet.createAccount("Alice");

        // Print the account.
        System.out.println(a);
    }
}
```

# Documentation

Please visit the [Shimmer Wiki](https://wiki.iota.org/shimmer/wallet.rs/welcome) for more information on using the IOTA Java Wallet Library.
More examples on how to use the library can be found [here](examples/src/main).