// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::account::AccountHandle;

pub(crate) use iota_migration::{
    client::{
        migration::{create_migration_bundle, mine, sign_migration_bundle, Address as MigrationAddress},
        response::InputData,
    },
    signing::ternary::seed::Seed as TernarySeed,
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

/// Migration data.
#[derive(Debug, Clone)]
pub struct MigrationData {
    /// Total seed balance.
    pub balance: u64,
    /// Migration inputs.
    pub inputs: Vec<InputData>,
}

/// Finds account data for the migration from legacy network.
pub struct MigrationDataFinder<'a> {
    pub(crate) node: &'a str,
    seed: TernarySeed,
    pub(crate) seed_hash: u64,
    pub(crate) security_level: u8,
    gap_limit: u64,
    initial_address_index: u64,
}

impl<'a> MigrationDataFinder<'a> {
    /// Creates a new migration accoutn data finder.
    pub fn new(node: &'a str, seed: &'a str) -> crate::Result<Self> {
        let mut hasher = DefaultHasher::new();
        seed.hash(&mut hasher);
        let seed_hash = hasher.finish();
        let seed = TernarySeed::from_trits(
            TryteBuf::try_from_str(&seed)
                .map_err(|_| crate::Error::InvalidSeed)?
                .as_trits()
                .encode::<T1B1Buf>(),
        )
        .map_err(|_| crate::Error::InvalidSeed)?;
        Ok(Self {
            node,
            seed,
            seed_hash,
            security_level: 2,
            gap_limit: 30,
            initial_address_index: 0,
        })
    }

    /// Sets the security level.
    pub fn with_security_level(mut self, level: u8) -> Self {
        self.security_level = level;
        self
    }

    /// Sets the initial address index.
    pub fn with_initial_address_index(mut self, initial_address_index: u64) -> Self {
        self.initial_address_index = initial_address_index;
        self
    }

    pub(crate) async fn finish(&self, inputs: &mut HashMap<Range<u64>, Vec<InputData>>) -> crate::Result<u64> {
        let mut address_index = self.initial_address_index;
        let mut legacy_client = iota_migration::ClientBuilder::new().node(self.node)?.build()?;
        let mut balance = 0;

        loop {
            let migration_inputs = legacy_client
                .get_account_data_for_migration()
                .with_seed(&self.seed)
                .with_security(self.security_level)
                .with_start_index(address_index)
                .with_gap_limit(self.gap_limit)
                .finish()
                .await?;
            let mut current_inputs = migration_inputs.1;
            // Filter duplicates because when it's called another time it could return duplicated entries
            let mut unique_inputs = HashMap::new();
            for input in current_inputs {
                unique_inputs.insert(input.index, input);
            }
            current_inputs = unique_inputs
                .into_iter()
                .map(|(_, input)| input)
                .collect::<Vec<InputData>>();
            let current_balance: u64 = current_inputs.iter().map(|d| d.balance).sum();
            balance += current_balance;
            inputs.insert(address_index..address_index + self.gap_limit, current_inputs);

            // if balance didn't change, we stop searching for balance
            if current_balance == 0 {
                break;
            }
            address_index += self.gap_limit;
        }

        Ok(balance)
    }
}

pub(crate) async fn create_bundle<P: AsRef<Path>>(
    account_handle: AccountHandle,
    data: &super::CachedMigrationData,
    seed: TernarySeed,
    address_inputs: Vec<&InputData>,
    bundle_mine: bool,
    timeout: Duration,
    log_file_path: P,
) -> crate::Result<Vec<BundledTransaction>> {
    let legacy_client = iota_migration::ClientBuilder::new().node(&data.node)?.build()?;

    match address_inputs.len() {
        0 => return Err(crate::Error::EmptyInputList),
        1 => {}
        _ if address_inputs.iter().any(|input| input.spent) => return Err(crate::Error::SpentAddressOnBundle),
        _ => {}
    }

    let deposit_address = account_handle.latest_address().await;
    let deposit_address = match MigrationAddress::try_from_bech32(&deposit_address.address().to_bech32()) {
        Ok(MigrationAddress::Ed25519(a)) => a,
        _ => return Err(crate::Error::InvalidAddress),
    };

    let mut prepared_bundle = create_migration_bundle(
        &legacy_client,
        deposit_address,
        address_inputs.clone().into_iter().cloned().collect(),
    )
    .await?;
    if bundle_mine && address_inputs.iter().any(|i| i.spent) {
        let mut spent_bundle_hashes = Vec::new();
        for input in &address_inputs {
            if let Some(bundle_hashes) = input.spent_bundlehashes.clone() {
                spent_bundle_hashes.extend(bundle_hashes);
            }
        }
        let mining_result = mine(
            prepared_bundle,
            data.security_level,
            false,
            spent_bundle_hashes,
            timeout.as_secs(),
        )
        .await?;
        prepared_bundle = mining_result.1;
    }

    let bundle = sign_migration_bundle(
        seed,
        prepared_bundle,
        address_inputs.clone().into_iter().cloned().collect(),
    )?;

    let mut log = OpenOptions::new().write(true).create(true).open(log_file_path)?;
    for i in 0..bundle.len() {
        let mut trits = TritBuf::<T1B1Buf>::zeros(8019);
        bundle.get(i).unwrap().as_trits_allocated(&mut trits);
        log.write_all(
            trits
                .encode::<T3B1Buf>()
                .iter_trytes()
                .map(char::from)
                .collect::<String>()
                .as_bytes(),
        )?;
        log.write_all(b"\n")?;
    }
    log.write_all(b"\n\n")?;

    Ok(bundle)
}

pub(crate) async fn send_bundle(node: &str, bundle: Vec<BundledTransaction>, mwm: u8) -> crate::Result<()> {
    let legacy_client = iota_migration::ClientBuilder::new().node(node)?.build()?;
    let _send_trytes = legacy_client
        .send_trytes()
        .with_trytes(bundle)
        .with_depth(2)
        .with_min_weight_magnitude(mwm)
        .finish()
        .await?;
    Ok(())
}
