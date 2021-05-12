## Migration example step by step breakdown

This document provides an overview of the nodejs migration example. The migration example serves as a reference for migrating IOTA tokens from the legacy network to new chrysalis network. 

The example specifies some constant values that are required for the migration process. The following table explains the purpose of each one:

| Name                      | Description                                                                                                                                                                                                                                                                                                             |
|---------------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| ADDRESS_SECURITY_LEVEL    | Set to 2. This is the security level for the addresses derived from the legacy seed. Possible values could be 1 | 2 | 3.                                                                                                                                                                                                |
| MINIMUM_MIGRATION_BALANCE | Set to 1000000 (1 Mi). The network will not accept any migration bundle that has accumulative balance less than 1 Mi.                                                                                                                                                                                                   |
| MAX_INPUTS_PER_BUNDLE     | Set to 10. Restricts the number of addresses / inputs that can fit into a migration bundle. Bundles with large number of inputs have low probability of getting easily confirmed. Note that this restriction may leave some addresses / inputs (with <1Mi) unmigrated (considering the MINIMUM_MIGRATION_BALANCE limit) |
| DB_STORAGE_PATH           | Set to `./migration-database`. The storage path where the new seed and their associated history will be stored.                                                                                                                                                                                                         |
| LEGACY_NETWORK_NODES      | Set to `['https://nodes.iota.org']`. This is the list of legacy network nodes where the migration bundles will be submitted. To run this example on a different network, the value for this constant needs to be changed.                                                                                               |
| LEGACY_PERMANODE          | Set to `https://chronicle.iota.org/api`. A permanode contains all historical data of IOTA transactions since genesis. This is needed to fetch old associated history against your legacy seed's addresses. Note that when running this example on a different (test) network, this can be set to `undefined`.           |
| CHRYSALIS_NODE            | Set to `https://chrysalis-nodes.iota.cafe`. The node for new chrysalis network.                                                                                                                                                                                                                                         |

The example also relies on some environment variables:

| Name                      | Description                                                                                                                                                                                                                                                                                                             |
|---------------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| SH_PASSWORD    | Password for stronghold. Stronghold contains the seed information for the new network.                                                                                                                                                                      |
| MIGRATION_SEED | Legacy IOTA seed from which the funds need to be migrated from.                                                                                                                                                                                                   |

The migration example implements all migration logic in a `run()` function. The rest of the document will break down that method for clarity.


---

```
    let migrationBundleHashes = [];
```

An internal variable that keeps tracks of the migrated bundle hashes. This is needed to monitor the confirmation states of the migrated bundles. 

---
 

```
    const callback = function (err, data) {
 
      if (data.event.type === 'TransactionConfirmed') {
        console.log("MigrationProgress:", data)
        migrationBundleHashes = migrationBundleHashes.filter(hash => hash !== data.event.data.bundleHash)

        if (migrationBundleHashes.length == 0) {
          process.exit()
        }
      }
    }

    addEventListener("MigrationProgress", callback)
```

Consume an event listener provided by the wallet library. This will essentially keep on checking the confirmation states of the migrated bundles and will only exit the program (example) when all migrated bundles are confirmed. 

---

```
    const manager = new AccountManager({
      storagePath: DB_STORAGE_PATH,
    })
```

Instantiate a account manager with the defined storage path. The manager instance will allow creating a mnemonic for the new network and will also allow to manipulate several other actions required for migration process.



---

```
    manager.setStrongholdPassword(process.env.SH_PASSWORD)
```

Set a strong password for stronghold file. This file will store the mnemonic generated in the next step. 



---

```
    const mnemonic = manager.generateMnemonic()
```

Generate a mnemonic phrase for the new network. This mnemonic derives accounts and addresses on the new network. 

:warning:**MAKE SURE TO BACK UP THIS MNEMONIC SECURELY AND ALSO VERIFY THAT YOU HAVE CORRECTLY COPIED / BACKED UP THE MNEMONIC. IF YOU LOSE THIS MNEMONIC PHRASE, YOU WILL NOT BE ABLE TO ACCESS YOUR FUNDS ON THE NEW NETWORK**:warning:



---

```
    manager.storeMnemonic(SignerType.Stronghold, mnemonic)
```

Store mnemonic in the stronghold file. This is needed to further execute actions on the mnemonic e.g., deriving accounts and addresses



---

```
    const account = await manager.createAccount({
      clientOptions: { node: CHRYSALIS_NODE, localPow: true, network: "chrysalis-mainnet" },
      alias: 'Migration',
    })

```

Create an account. After this is created, the wallet library will automatically pick addresses from this account to receive the migrated funds.



---

```
    const migrationData = await manager.getMigrationData(
      nodes,
      seed,
      {
        permanode: LEGACY_PERMANODE,
        securityLevel: ADDRESS_SECURITY_LEVEL,
        initialAddressIndex: 0
      }
    )
```

This step searches the legacy network (and permanode if provided) and gets associated history (addresses, balances). An important thing to note here is `initialAddressIndex` which defines the starting search index for the addresses. If you expect funds on your seed and do not get the correct balance, you would probably need to re-run the script with a higher address index. 



---

```
      let input_batches = getMigrationBundles(migrationData.inputs)
```

This step prepares migration bundles. Addresses with spent inputs / addresses are not grouped together with other inputs as they have to go through a bundle mining process. Rest of the unspent addresses are grouped together based on the `MINIMUM_MIGRATION_BALANCE` and `MAX_INPUTS_PER_BUNDLE` restrictions. 



---

```
 for (batch of input_batches) {
        try {
          const bundle = await manager.createMigrationBundle(seed, batch.inputs.map(input => input.index), {
            logFileName: 'iota-migration.log',
            mine: batch.inputs[0].spent
          })
          migrationBundleHashes.push(bundle.bundleHash)
        } catch (e) {
          console.error(e);
        }
      }
```

After preparation of migration bundles, this step iterates on each migration bundle and signs it. Note that if any migration bundle has spent address, it will have to go through a bundle mining process that will take 10 minutes for each bundle. 



---

```
 for (bundleHash of migrationBundleHashes) {
        try {
          await manager.sendMigrationBundle(nodes, bundleHash)
        } catch (e) { console.error(e) }
      }
```

The final step is to broadcast the migrated bundles to the network. The above piece of code will broadcast each signed migration bundle to the network. 

---

Note that even after broadcast the example will continue to run as it is monitoring the confirmation status of each broadcast bundle. It will only exit once all the migration bundles are confirmed. Also, once this process completes, there will be a log file generated that will contain the information about the bundles sent to the network alongwith the information about addresses expected to receive the funds on the new network. 
