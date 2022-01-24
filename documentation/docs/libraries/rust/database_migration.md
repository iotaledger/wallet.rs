## Database migration guide

The implemetion is based on original ideas described [here](https://github.com/Thoralf-M/wallet-core/blob/dev/documentation/database_migration.md) by Thoralf.

To create a new schema, a new module with the corresponding schema version should be created in the `schema` module, in this case it's `v2`. The sample code snippet shows a sample implementation from `schema::v1::Account` to `schema::v2::Account`. `schema::v2::Account` renames field `something` to `another`, introduces a new field `value` with a default value of 5, and not immediately obvious by perusing `schema::v2::Account` is the change in `ClientOptions` struct, primary_node will no longer be de/serialized as 'node' but as 'primaryNode', network is renamed to network_id and will be de/serialized as 'networkId'.

New schema versions reuse unchanged struct and enum types from previous schema to save time, effort and possible bugs in converting between types that are identical but have different parent modules.

Although not shown here, schema version cannot be immediately set for newly created stronghold files until stronghold password is set, the field for persisting schema version number is also backed up and used for migration when importing accounts.

Also notice how `Url` is represented as a `String`, the idea is to reduce dependency by keeping field types that aren't objects and arrays as either a number or a string.

To achieve incremental migration across multiple versions, every new schema version must implement `From` trait conversion for its previous schema, for example, `schema::v2::Account` implements `From<schema::v1::Account>` and so on.

- main.rs
```rust
// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::str::FromStr;

use serde;
use url::Url;

/// This is the public interface. It must match the latest Account version in the schema module.
#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Node {
    /// Node url.
    pub url: Url,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ClientOptions {
    primary_node: Option<Node>,
    network_id: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Account {
    pub another: u64,
    pub value: u64,
    pub client_options: ClientOptions,
}

mod schema {
    use iota_wallet::Result;
    use std::ops::RangeInclusive;

    pub mod v1 {
        type Url = String;

        #[derive(Debug, serde::Serialize, serde::Deserialize)]
        #[serde(deny_unknown_fields)]
        pub struct Node {
            /// Node url.
            pub url: Url,
        }

        #[derive(Debug, serde::Serialize, serde::Deserialize)]
        #[serde(rename_all = "camelCase", deny_unknown_fields)]
        pub struct ClientOptions {
            #[serde(rename = "node")] // here just for DB compatibility; can be changed when migrations are implemented
            pub primary_node: Option<Node>,
            pub network: Option<String>,
        }

        #[derive(serde::Serialize, serde::Deserialize)]
        #[serde(rename_all = "camelCase", deny_unknown_fields)]
        pub struct Account {
            pub something: u64,
            pub client_options: ClientOptions,
        }
    }

    mod v2 {
        pub type Node = super::v1::Node;    // Node struct hasn't changed so we can reuse

        #[derive(Debug, serde::Serialize, serde::Deserialize)]
        #[serde(rename_all = "camelCase", deny_unknown_fields)]
        pub struct ClientOptions {
            pub primary_node: Option<Node>,
            pub network_id: Option<String>,
        }

        impl From<super::v1::ClientOptions> for ClientOptions {
            fn from(client_options: super::v1::ClientOptions) -> Self {
                ClientOptions {
                    primary_node: client_options.primary_node,
                    network_id: client_options.network,
                }
            }
        }

        #[derive(serde::Serialize, serde::Deserialize)]
        #[serde(rename_all = "camelCase", deny_unknown_fields)]
        pub struct Account {
            pub another: u64,
            pub value: u64,
            client_options: ClientOptions,
        }

        impl From<super::v1::Account> for Account {
            // converts the v1 account schema to the v2 schema
            // here you can do anything, pull info from the node, etc
            fn from(account: super::v1::Account) -> Self {
                Account {
                    another: account.something,
                    value: 5,
                    client_options: account.client_options.into(),
                }
            }
        }
    }

    /// Version provides easy abstraction for handling different schema versions as the same type during migration
    #[derive(serde::Serialize)]
    #[serde(untagged)]
    enum Version {
        V1(v1::Account),
        V2(v2::Account),
    }

    impl Version {
        /// Create a Version variant from provided data string
        fn new(version: usize, data: String) -> Result<Version> {
            match version {
                1 => {
                    let account: v1::Account = match serde_json::from_str(&data) {
                        Ok(account) => account,
                        Err(e) => panic!("Couldn't parse schema version '{}' data: '{}'; {:?}", version, data, e),
                    };
                    Ok(Version::V1(account))
                }
                version => {
                    panic!("Couldn't convert data into Version type; Please implement Version variant and conversion for schema version number `{}`", version);
                }
            }
        }

        /// Consume variant and create a new variant for the next schema version
        fn next_version(self) -> Result<Self> {
            match self {
                Version::V1(account) => Ok(Version::V2(account.into())),
                _ => {
                    let version_number = self.number();
                    panic!(
                        "Version conversion from `{}` to `{}` not implemented",
                        version_number,
                        version_number + 1
                    );
                }
            }
        }

        /// Return schema version number for variant
        fn number(&self) -> usize {
            match &self {
                Version::V1(_) => 1,
                Version::V2(_) => 2,
            }
        }
    }

    /// Migrate data incrementally across versions provided in `range`
    /// This implementation is generic and doesn't need to change as new schema versions get added
    pub(crate) fn migrate(range: RangeInclusive<usize>, data: String) -> Result<String> {
        let (db_version, target_version) = if range.is_empty() {
            panic!("Schema range invalid: `{:?}`;", range);
        } else {
            (*range.start(), *range.end())
        };

        let mut version = Some(Version::new(db_version, data)?);

        for _ in range.skip(1) {
            if let Some(v) = version.take() {
                version = Some(v.next_version()?);
            }
        }

        debug_assert_eq!(version.as_ref().unwrap().number(), target_version);

        Ok(serde_json::to_string(&version).unwrap())
    }
}

#[tokio::main]
async fn main() -> iota_wallet::Result<()> {
    let primary_node = "https://api.lb-1.h.chrysalis-devnet.iota.cafe/";
    let network_id = "chrysalis-devnet";

    let account_v1 = schema::v1::Account {
        something: 64,
        client_options: schema::v1::ClientOptions {
            primary_node: Some(schema::v1::Node {
                url: primary_node.to_string(),
            }),
            network: Some(network_id.to_string()),
        },
    };

    let data = serde_json::to_string(&account_v1).unwrap();
    println!("Version 1 -> {}", data);

    let data = schema::migrate(1..=2, data).unwrap();
    println!("Version 2 -> {}", data);
    
    let account: Account = serde_json::from_str(&data).unwrap();
    assert_eq!(account.another, 64);
    assert_eq!(account.value, 5);
    assert_eq!(account.client_options.primary_node.unwrap().url, Url::from_str(primary_node).unwrap());
    assert_eq!(account.client_options.network_id, Some(network_id.to_string()));

    Ok(())
}

```
