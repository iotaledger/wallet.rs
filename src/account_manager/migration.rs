// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::AccountHandle,
    event::{emit_migration_progress, MigrationProgressType},
};

use chrono::prelude::Utc;
use serde::{Deserialize, Serialize};

use iota_migration::crypto::hashes::ternary::{curl_p::CurlP, Hash as TernaryHash};
pub(crate) use iota_migration::{
    client::{
        migration::{
            create_migration_bundle, encode_migration_address, mine, sign_migration_bundle, Address as BeeAddress,
        },
        response::InputData,
    },
    crypto::keys::ternary::seed::Seed as TernarySeed,
    ternary::{T1B1Buf, T3B1Buf, TritBuf, TryteBuf},
    transaction::bundled::{BundledTransaction, BundledTransactionField},
};

use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    fs::OpenOptions,
    hash::{Hash, Hasher},
    io::Write,
    ops::Range,
    path::Path,
    time::Duration,
};

/// Migration address
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationAddress {
    /// address tryte encoded
    pub trytes: String,
    /// address bech32 encoded
    pub bech32: String,
}

/// Migration data.
#[derive(Debug, Clone)]
pub struct MigrationData {
    /// Total seed balance.
    pub balance: u64,
    /// The index of the last checked address.
    /// Useful if you want to call the finder again.
    pub last_checked_address_index: u64,
    /// Migration inputs.
    pub inputs: Vec<InputData>,
    /// If any of the inputs are spent
    pub spent_addresses: bool,
}

/// Migration bundle.
#[derive(Debug, Clone)]
pub struct MigrationBundle {
    /// The bundle crackability if it was mined.
    pub crackability: f64,
    /// Migration bundle.
    pub bundle: Vec<BundledTransaction>,
}

/// Finds account data for the migration from legacy network.
pub struct MigrationDataFinder<'a> {
    pub(crate) nodes: &'a [&'a str],
    pub(crate) permanode: Option<&'a str>,
    seed: TernarySeed,
    pub(crate) seed_hash: u64,
    pub(crate) security_level: u8,
    pub(crate) gap_limit: u64,
    pub(crate) initial_address_index: u64,
}

/// Migration metadata.
pub(crate) struct MigrationMetadata {
    pub(crate) balance: u64,
    pub(crate) last_checked_address_index: u64,
    pub(crate) inputs: HashMap<Range<u64>, Vec<InputData>>,
    pub(crate) spent_addresses: bool,
}

#[derive(Serialize)]
struct LogAddress {
    address: String,
    balance: u64,
}

impl<'a> MigrationDataFinder<'a> {
    /// Creates a new migration accoutn data finder.
    pub fn new(nodes: &'a [&'a str], seed: &'a str) -> crate::Result<Self> {
        let mut hasher = DefaultHasher::new();
        seed.hash(&mut hasher);
        let seed_hash = hasher.finish();
        let seed = TernarySeed::from_trits(
            TryteBuf::try_from_str(seed)
                .map_err(|_| crate::Error::InvalidSeed)?
                .as_trits()
                .encode::<T1B1Buf>(),
        )
        .map_err(|_| crate::Error::InvalidSeed)?;
        Ok(Self {
            nodes,
            permanode: None,
            seed,
            seed_hash,
            security_level: 2,
            gap_limit: 30,
            initial_address_index: 0,
        })
    }

    /// Sets the permanode to use.
    pub fn with_permanode(mut self, permanode: &'a str) -> Self {
        self.permanode.replace(permanode);
        self
    }

    /// Sets the security level.
    pub fn with_security_level(mut self, level: u8) -> Self {
        self.security_level = level;
        self
    }

    /// Sets the gap limit.
    pub fn with_gap_limit(mut self, gap_limit: u64) -> Self {
        self.gap_limit = gap_limit;
        self
    }

    /// Sets the initial address index.
    pub fn with_initial_address_index(mut self, initial_address_index: u64) -> Self {
        self.initial_address_index = initial_address_index;
        self
    }

    pub(crate) async fn finish(
        &self,
        previous_inputs: HashMap<Range<u64>, Vec<InputData>>,
    ) -> crate::Result<MigrationMetadata> {
        let mut inputs: HashMap<Range<u64>, Vec<InputData>> = HashMap::new();
        let mut address_index = self.initial_address_index;
        let mut legacy_client_builder = iota_migration::ClientBuilder::new().quorum(true);
        if let Some(permanode) = self.permanode {
            legacy_client_builder = legacy_client_builder.permanode(permanode)?;
        }
        for node in self.nodes {
            legacy_client_builder = legacy_client_builder.node(node)?;
        }
        let mut legacy_client = legacy_client_builder.build()?;
        let mut balance = 0;
        let mut spent_addresses = false;
        loop {
            emit_migration_progress(MigrationProgressType::FetchingMigrationData {
                initial_address_index: address_index,
                final_address_index: address_index + self.gap_limit,
            })
            .await;
            let migration_inputs = legacy_client
                .get_account_data_for_migration()
                .with_seed(&self.seed)
                .with_security(self.security_level)
                .with_start_index(address_index)
                .with_gap_limit(self.gap_limit)
                .finish()
                .await?;
            if migration_inputs.2 {
                spent_addresses = true;
            }
            let mut current_inputs = migration_inputs.1;
            // Filter duplicates because when it's called another time it could return duplicated entries
            let mut unique_inputs = HashMap::new();
            for input in current_inputs {
                let mut exists = false;
                // check inputs on previous executions
                for previous_inputs in previous_inputs.values() {
                    if previous_inputs.contains(&input) {
                        exists = true;
                        break;
                    }
                }
                // check inputs on previous iterations
                if !exists {
                    for previous_inputs in inputs.values() {
                        if previous_inputs.contains(&input) {
                            exists = true;
                            break;
                        }
                    }
                }
                if !exists {
                    unique_inputs.insert(input.index, input);
                }
            }
            current_inputs = unique_inputs.into_values().collect::<Vec<InputData>>();
            let current_balance: u64 = current_inputs.iter().map(|d| d.balance).sum();
            balance += current_balance;
            inputs.insert(address_index..address_index + self.gap_limit, current_inputs);

            address_index += self.gap_limit;
            // if balance didn't change, we stop searching for balance
            if current_balance == 0 {
                break;
            }
        }

        Ok(MigrationMetadata {
            balance,
            last_checked_address_index: address_index,
            inputs,
            spent_addresses,
        })
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn create_bundle<P: AsRef<Path>>(
    account_handle: AccountHandle,
    data: &super::CachedMigrationData,
    seed: TernarySeed,
    address_inputs: Vec<&InputData>,
    bundle_mine: bool,
    timeout: Duration,
    offset: i64,
    log_file_path: P,
) -> crate::Result<MigrationBundle> {
    let mut legacy_client_builder = iota_migration::ClientBuilder::new().quorum(true);
    if let Some(permanode) = &data.permanode {
        legacy_client_builder = legacy_client_builder.permanode(permanode)?;
    }
    for node in &data.nodes {
        legacy_client_builder = legacy_client_builder.node(node)?;
    }
    let legacy_client = legacy_client_builder.build()?;

    match address_inputs.len() {
        0 => return Err(crate::Error::EmptyInputList),
        1 => {}
        _ if address_inputs.iter().any(|input| input.spent) => return Err(crate::Error::SpentAddressOnBundle),
        _ => {}
    }

    let deposit_address = account_handle.latest_address().await;
    let deposit_address_bech32 = deposit_address.address().to_bech32();
    let deposit_address = match BeeAddress::try_from_bech32(&deposit_address.address().to_bech32()) {
        Ok(BeeAddress::Ed25519(a)) => a,
        _ => return Err(crate::Error::InvalidAddress),
    };
    let deposit_address_trytes = encode_migration_address(deposit_address)?;

    let mut prepared_bundle = create_migration_bundle(
        &legacy_client,
        deposit_address,
        address_inputs.clone().into_iter().cloned().collect(),
    )
    .await?;
    let mut crackability = None;
    let mut spent_bundle_hashes = Vec::new();
    if bundle_mine && address_inputs.iter().any(|i| i.spent) {
        for input in &address_inputs {
            if let Some(bundle_hashes) = input.spent_bundlehashes.clone() {
                spent_bundle_hashes.extend(bundle_hashes);
            }
        }
        if !spent_bundle_hashes.is_empty() {
            emit_migration_progress(MigrationProgressType::MiningBundle {
                address: address_inputs
                    .iter()
                    .find(|i| i.spent)
                    .unwrap() // safe to unwrap: we checked that there's an spent address
                    .address
                    .to_inner()
                    .encode::<T3B1Buf>()
                    .iter_trytes()
                    .map(char::from)
                    .collect::<String>(),
            })
            .await;
            let mining_result = mine(
                prepared_bundle,
                data.security_level,
                spent_bundle_hashes.clone(),
                timeout.as_secs(),
                offset,
            )
            .await?;
            crackability = Some(mining_result.0.crackability);
            prepared_bundle = mining_result.1;
        }
    }

    emit_migration_progress(MigrationProgressType::SigningBundle {
        addresses: address_inputs
            .iter()
            .map(|i| {
                i.address
                    .to_inner()
                    .encode::<T3B1Buf>()
                    .iter_trytes()
                    .map(char::from)
                    .collect::<String>()
            })
            .collect(),
    })
    .await;
    let bundle = sign_migration_bundle(
        seed,
        prepared_bundle,
        address_inputs.clone().into_iter().cloned().collect(),
    )?;

    let bundle_hash = bundle
        .first()
        .unwrap()
        .bundle()
        .to_inner()
        .encode::<T3B1Buf>()
        .iter_trytes()
        .map(char::from)
        .collect::<String>();

    let mut log = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(log_file_path)?;
    let mut trytes = Vec::new();
    for i in 0..bundle.len() {
        let mut trits = TritBuf::<T1B1Buf>::zeros(8019);
        bundle.get(i).unwrap().as_trits_allocated(&mut trits);
        trytes.push(
            trits
                .encode::<T3B1Buf>()
                .iter_trytes()
                .map(char::from)
                .collect::<String>(),
        );
    }
    log.write_all(format!("bundleHash: {}\n", bundle_hash).as_bytes())?;
    log.write_all(format!("trytes: {:?}\n", trytes).as_bytes())?;
    log.write_all(
        format!(
            "receiveAddressTrytes: {}\n",
            deposit_address_trytes
                .to_inner()
                .encode::<T3B1Buf>()
                .iter_trytes()
                .map(char::from)
                .collect::<String>()
        )
        .as_bytes(),
    )?;
    log.write_all(format!("receiveAddressBech32: {}\n", deposit_address_bech32).as_bytes())?;
    log.write_all(format!("balance: {}\n", address_inputs.iter().map(|a| a.balance).sum::<u64>()).as_bytes())?;
    log.write_all(format!("timestamp: {}\n", Utc::now()).as_bytes())?;
    log.write_all(
        format!(
            "spentAddresses: {:?}\n",
            address_inputs
                .iter()
                .filter(|i| i.spent)
                .map(|i| serde_json::to_string_pretty(&LogAddress {
                    address: i
                        .address
                        .to_inner()
                        .encode::<T3B1Buf>()
                        .iter_trytes()
                        .map(char::from)
                        .collect::<String>(),
                    balance: i.balance
                })
                .unwrap())
                .collect::<Vec<String>>()
        )
        .as_bytes(),
    )?;
    let spent_bundle_hashes = match spent_bundle_hashes.is_empty() {
        false => format!("{:?}", spent_bundle_hashes),
        true => "null".to_string(),
    };
    log.write_all(format!("spentBundleHashes: {}\n", spent_bundle_hashes).as_bytes())?;
    log.write_all(format!("mine: {}\n", bundle_mine).as_bytes())?;
    log.write_all(
        format!(
            "crackability: {}\n",
            if let Some(crackability) = crackability {
                crackability.to_string()
            } else {
                "null".to_string()
            }
        )
        .as_bytes(),
    )?;
    log.write_all(b"\n\n")?;

    Ok(MigrationBundle {
        crackability: crackability.unwrap_or_default(),
        bundle,
    })
}

pub(crate) async fn send_bundle(
    nodes: &[&str],
    bundle: Vec<BundledTransaction>,
    mwm: u8,
) -> crate::Result<iota_migration::crypto::hashes::ternary::Hash> {
    let mut builder = iota_migration::ClientBuilder::new();
    for node in nodes {
        builder = builder.node(node)?;
    }
    let legacy_client = builder.build()?;

    let bundle_hash = *bundle.first().unwrap().bundle();
    let bundle_hash_string = bundle_hash
        .to_inner()
        .encode::<T3B1Buf>()
        .iter_trytes()
        .map(char::from)
        .collect::<String>();

    emit_migration_progress(MigrationProgressType::BroadcastingBundle {
        bundle_hash: bundle_hash_string.clone(),
    })
    .await;

    let send_trytes = legacy_client
        .send_trytes()
        .with_trytes(bundle)
        .with_depth(2)
        .with_min_weight_magnitude(mwm)
        .finish()
        .await?;

    tokio::spawn(async move {
        loop {
            if let Ok(r) = check_confirmation(&legacy_client, &bundle_hash, &bundle_hash_string).await {
                if r {
                    break;
                }
            }
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        }
    })
    .await?;
    let mut trits = TritBuf::<T1B1Buf>::zeros(BundledTransaction::trit_len());
    let mut curl = CurlP::new();
    send_trytes[0].as_trits_allocated(&mut trits);
    let tail_transaction_hash = TernaryHash::from_inner_unchecked(curl.digest(&trits));
    Ok(tail_transaction_hash)
}

async fn check_confirmation(
    legacy_client: &iota_migration::Client,
    bundle_hash: &iota_migration::crypto::hashes::ternary::Hash,
    bundle_hash_string: &str,
) -> crate::Result<bool> {
    log::debug!("[MIGRATION] checking confirmation for bundle `{}`", bundle_hash_string);
    let hashes = legacy_client
        .find_transactions()
        .bundles(&[*bundle_hash])
        .send()
        .await?
        .hashes;
    let transactions = legacy_client.get_trytes(&hashes).await?.trytes;
    let mut infos = Vec::new();
    for transaction in transactions {
        if transaction.is_tail() {
            let mut trits = TritBuf::<T1B1Buf>::zeros(BundledTransaction::trit_len());
            let mut curl = CurlP::new();
            transaction.as_trits_allocated(&mut trits);
            let tail_transaction_hash = TernaryHash::from_inner_unchecked(curl.digest(&trits));
            let info = legacy_client.get_tip_info(&tail_transaction_hash).await?;
            infos.push((tail_transaction_hash, info));
        }
    }

    // check if the transaction was confirmed
    if infos.iter().any(|(_, i)| i.confirmed) {
        log::debug!("[MIGRATION] bundle `{}` confirmed", bundle_hash_string);
        emit_migration_progress(MigrationProgressType::TransactionConfirmed {
            bundle_hash: bundle_hash_string.to_string(),
        })
        .await;
        return Ok(true);
    }

    // Check if there exists a non-lazy tip (requiring no promotion or reattachment)
    if let Some((hash, _)) = infos
        .iter()
        .find(|(_, i)| !i.should_promote && !i.should_reattach && !i.conflicting)
    {
        log::debug!(
            "[MIGRATION] bundle `{}` has a non-lazy tip, tail transaction hash: `{}`",
            bundle_hash_string,
            hash.to_inner()
                .encode::<T3B1Buf>()
                .iter_trytes()
                .map(char::from)
                .collect::<String>()
        );
    } else {
        for (tail_transaction_hash, info) in infos {
            if info.should_reattach {
                log::debug!(
                    "[MIGRATION] reattaching bundle `{}`, tail transaction hash: `{}`",
                    bundle_hash_string,
                    tail_transaction_hash
                        .to_inner()
                        .encode::<T3B1Buf>()
                        .iter_trytes()
                        .map(char::from)
                        .collect::<String>()
                );
                legacy_client.reattach(&tail_transaction_hash).await?.finish().await?;
                break;
            }
        }
    }

    Ok(false)
}
