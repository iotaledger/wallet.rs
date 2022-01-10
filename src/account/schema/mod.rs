// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

/// Schema module for incremental migration of database
use crate::Error;
use serde::Serialize;
use std::ops::RangeInclusive;

mod v1;

/// Version provides easy abstraction for handling different schema versions as the same type during migration
#[derive(Serialize)]
#[serde(untagged)]
enum Version {
    V1(v1::Account),
}

impl Version {
    /// Create a Version variant from provided data string
    fn new(version: usize, data: String) -> crate::Result<Version> {
        match version {
            1 => {
                let account: v1::Account = serde_json::from_str(&data).map_err(|e| {
                    Error::AccountSchemaMigrationError(format!("Couldn't parse schema version '{}' data: '{}'; {:?}", version, data, e))
                })?;
                Ok(Version::V1(account))
            }
            version => {
                return Err(Error::AccountSchemaMigrationError(format!(
                    "Couldn't convert data into Version type; Please implement Version variant and conversion for schema version number `{}`",
                    version
                )))
            }
        }
    }

    /// Consume variant and create a new variant for the next schema version
    fn next_version(self) -> crate::Result<Self> {
        match self {
            Version::V1(_) => {
                let version_number = self.number();
                return Err(Error::AccountSchemaMigrationError(format!(
                    "Version conversion from `{}` to `{}` not implemented",
                    version_number,
                    version_number + 1
                )));
            }
        }
    }

    /// Return schema version number for variant
    fn number(&self) -> usize {
        match &self {
            Version::V1(_) => 1,
        }
    }
}

/// Migrate data incrementally across versions provided in `range`
/// This implementation is generic and doesn't need to change as new schema versions get added
pub(crate) fn migrate(range: RangeInclusive<usize>, data: String) -> crate::Result<String> {
    let (db_version, target_version) = if range.is_empty() {
        return Err(crate::Error::AccountSchemaMigrationError(format!(
            "Schema range invalid: `{:?}`;",
            range
        )));
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

    serde_json::to_string(&version).map_err(crate::Error::JsonError)
}

#[cfg(test)]
mod tests {
    use crate::{
        account::{schema::v1, AccountHandle},
        address::{AddressOutput, OutputKind},
        client,
        client::{BrokerOptions, ClientOptionsBuilder},
    };

    use iota_client::bee_message::prelude::{MessageId, TransactionId};

    use std::{collections::HashMap, time::Duration};

    fn demo_client_options() -> crate::Result<client::ClientOptions> {
        let broker_options = BrokerOptions {
            automatic_disconnect: Some(false),
            timeout: Some(Duration::from_secs(30)),
            use_ws: Some(false),
            port: Some(80),
            max_reconnection_attempts: Some(4),
        };

        ClientOptionsBuilder::new()
            .with_primary_node_auth(
                "https://api.lb-1.h.chrysalis-devnet.iota.cafe",
                Default::default(),
                None,
            )?
            .with_primary_pow_node_auth(
                "https://api.lb-1.h.chrysalis-devnet.iota.cafe",
                Default::default(),
                None,
            )?
            .with_node_auth(
                "https://api.lb-0.h.chrysalis-devnet.iota.cafe",
                Default::default(),
                None,
            )?
            .with_node_auth(
                "https://api.lb-1.h.chrysalis-devnet.iota.cafe",
                Default::default(),
                None,
            )?
            .with_network("chrysalis-devnet")
            .with_mqtt_mqtt_broker_options(broker_options)
            .with_local_pow(true)
            .with_node_sync_interval(Duration::from_secs(300))
            .with_request_timeout(Duration::from_secs(30))
            .with_api_timeout(client::Api::GetTips, Duration::from_secs(60))
            .with_api_timeout(client::Api::GetBalance, Duration::from_secs(60))
            .build()
    }

    async fn update_address_balance(account_handle: &AccountHandle) {
        // update address balance so we can create the next account
        let mut account = account_handle.write().await;
        let mut outputs = HashMap::default();
        let output = AddressOutput {
            transaction_id: TransactionId::new([0; 32]),
            message_id: MessageId::new([0; 32]),
            index: 0,
            amount: 5,
            is_spent: false,
            address: crate::test_utils::generate_random_iota_address(),
            kind: OutputKind::SignatureLockedSingle,
        };
        outputs.insert(output.id().unwrap(), output);
        for address in account.addresses_mut() {
            address.set_outputs(outputs.clone());
        }
    }

    #[tokio::test]
    async fn validate_schema() {
        crate::test_utils::with_account_manager(crate::test_utils::TestType::Storage, |manager, _| async move {
            let account_handle = manager
                .create_account(demo_client_options().unwrap())
                .unwrap()
                .skip_persistence()
                .initialise()
                .await
                .expect("failed to add account");
            account_handle.generate_address().await.unwrap();
            update_address_balance(&account_handle).await;

            let account = account_handle.read().await.clone();

            let data = serde_json::to_string(&account).unwrap();

            match crate::account::LATEST_SCHEMA_VERSION {
                1 => assert!(serde_json::from_str::<v1::Account>(&data).is_ok()),
                version_number => panic!("Couldn't validate schema version `{}`", version_number),
            }
        })
        .await;
    }
}
