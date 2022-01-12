---
description: Official IOTA Client Library Java API examples.
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


For the rest of the examples in this document we will be using the `node()` method below:
```java
private static Client node() {
    String nodeUrl = "https://chrysalis-nodes.iota.cafe:443";
    Client iota = Client.Builder()
        // Insert your node URL here
        .withNode(nodeUrl) 
        // Or instead here but with authentication
        .withNodeAuth("https://somechrysalisiotanode.com", "jwt_or_null", "name_or_null", "password_or_null")
        // Choose pow mode
        .withLocalPow(true)
        // You can also set a time-out in seconds for the API calls
        .withRequestTimeout(5)
        //Then create the Client instance
        .finish();
    return iota;
}
```

***

The most basic example is creating a client, and then requesting the information about the node. 
```java

```

Example output of the code would be:
```bash

```

***

You can find more advanced examples in the [examples](https://github.com/iotaledger/wallet.rs/tree/dev/bindings/java/examples/java-app) folder.
