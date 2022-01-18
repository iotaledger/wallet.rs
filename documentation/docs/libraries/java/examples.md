---
description: Official IOTA Wallet Library Java API examples.
image: /img/logo/iota_mark_light.png
keywords:
- api
- Java
- examples
- type
- node
- client
---
# Examples

It's possible to send transactions with iota.rs, but we strongly recommend to use official `wallet.rs` library together with `stronghold.rs` enclave for value-based transfers. This combination incorporates the best security practices while dealing with seeds, related addresses and `UTXO`. See more information on [wallet docs](https://wiki.iota.org/wallet.rs/welcome).

```bash
git clone https://github.com/iotaledger/iota.rs
```

```bash
cd iota.rs/bindings/java
```

Examples are all collected in a sample project. By default it runs a node info example, but there are many more.

Run the example like:

Gradle: `./gradlew examples:java-app:test --info`

Maven: `cd examples/java-app && mvn test`


## Backup and Restore Example

1. Create an account manager and set a password:

```java
AccountManager manager = AccountManager.Builder().finish();

manager.setStrongholdPassword("password");
manager.storeMnemonic(AccountSignerType.STRONGHOLD, null);

```

2. Create your account:

```java
ClientOptions clientOptions = new ClientOptionsBuilder()
    .withNode("https://api.lb-0.h.chrysalis-devnet.iota.cafe")
    .build();
Account account = manager
    .createAccount(client_options)
    .alias("alias")
    .initialise();
String id = account.id();

```

3. You can secure your account in a backup file:
```java
// backup the stored accounts to ./backup/${backup_name}
Path backupPath = manager.backup("./backup");
```


4. You can import the backup later, or in another application using the following snippet:
```java
manager.importAccounts(backupPath, "password");

Account imported_account_handle = manager.getAccount(id);
```

That's it! You can now backup and restore your account!

## Transfer funds

1. Get or Create your account:
```java
AccountManager manager = AccountManager.Builder().finish();

manager.setStrongholdPassword("password");

// Get account or create a new one
String accountAlias = "alias";
Account account;
try {
    account = manager.getAccount(accountAlias)
} catch (WalletException e) {
    // first we'll create an example account and store it
    manager.storeMnemonic(AccountSignerType.STRONGHOLD, null);
    ClientOptions clientOptions = new ClientOptionsBuilder()
        .withNode("https://api.lb-0.h.chrysalis-devnet.iota.cafe")
        .build();
    account = manager
        .createAccount(client_options)
        .alias(accountAlias)
        .initialise();
}
```

2. Generate the address:
```java
Address address = account.generateAddress();
```

3. Print and wait
```java
System.out.println("Send iotas from the faucet to {} and press enter after the transaction got confirmed" +
    address
);

System.in.read();
```

4. Send and wait
```java
System.out.println("Sending transfer...");
Message message = account
    .transfer(
        Transfer.builder(
            AddressWrapper.parse("atoi1qzt0nhsf38nh6rs4p6zs5knqp6psgha9wsv74uajqgjmwc75ugupx3y7x0r"),
            10000000,
            OutputKind.SIGNATURE_LOCKED_DUST_ALLOWANCE),
        )
        .finish(),
    );
System.out.println("Message sent: " + message.id());
```

## Event listening

```java

```

***

You can find more advanced examples in the [examples](https://github.com/iotaledger/wallet.rs/tree/dev/bindings/java/examples/java-app) folder.
