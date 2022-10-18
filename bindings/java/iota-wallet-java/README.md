---
description: Get started with the official IOTA Wallet Java library.
image: /img/logo/iota_mark_light.png
keywords:

- Java
- jar
- Gradle
- Maven

---
# IOTA Wallet Java Library

Get started with the official IOTA Wallet Java Library.

## Requirements

* Make sure you have the latest [Java Development Kit (JDK)](https://www.oracle.com/java/technologies/downloads/) installed.

## Install the Library

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

## Use the Library

In order to use the library, you need to create a _Wallet_:

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

## What's Next?

Now that you are up and running, you can get acquainted with the library using
its [how-to guides](../how_tos/run_how_tos.mdx) and the
repository's [code examples](https://github.com/iotaledger/wallet.rs/tree/develop/bindings/java/iota-wallet-java/examples/src).