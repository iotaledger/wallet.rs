// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::account::Account;

use iota::{common::packable::Packable, ReferenceUnlock, UnlockBlock};

use std::{collections::HashMap, path::PathBuf};

#[derive(Default)]
pub struct StrongholdSigner;

fn stronghold_path(storage_path: &PathBuf) -> PathBuf {
    if storage_path.extension().unwrap_or_default() == "stronghold" {
        storage_path.clone()
    } else if storage_path.is_dir() {
        storage_path.join(crate::account_manager::STRONGHOLD_FILENAME)
    } else if let Some(parent) = storage_path.parent() {
        parent.join(crate::account_manager::STRONGHOLD_FILENAME)
    } else {
        storage_path.clone()
    }
}

#[async_trait::async_trait]
impl super::Signer for StrongholdSigner {
    async fn store_mnemonic(&self, storage_path: &PathBuf, mnemonic: String) -> crate::Result<()> {
        crate::stronghold::store_mnemonic(&stronghold_path(storage_path), mnemonic).await?;
        Ok(())
    }

    async fn generate_address(
        &self,
        account: &Account,
        address_index: usize,
        internal: bool,
    ) -> crate::Result<iota::Address> {
        let address = crate::stronghold::generate_address(
            &stronghold_path(account.storage_path()),
            *account.index(),
            address_index,
            internal,
        )
        .await?;
        Ok(address)
    }

    async fn sign_message(
        &self,
        account: &Account,
        essence: &iota::TransactionPayloadEssence,
        inputs: &mut Vec<super::TransactionInput>,
    ) -> crate::Result<Vec<iota::UnlockBlock>> {
        let serialized_essence = essence.pack_new();

        let mut unlock_blocks = vec![];
        let mut current_block_index: usize = 0;
        let mut signature_indexes = HashMap::<usize, usize>::new();
        inputs.sort_by(|a, b| a.input.cmp(&b.input));

        for recorder in inputs.iter() {
            // Check if current path is same as previous path
            // If so, add a reference unlock block
            if let Some(block_index) = signature_indexes.get(&recorder.address_index) {
                unlock_blocks.push(UnlockBlock::Reference(ReferenceUnlock::new(*block_index as u16)?));
            } else {
                // If not, we should create a signature unlock block
                let signature = crate::stronghold::sign_essence(
                    &stronghold_path(account.storage_path()),
                    serialized_essence.clone(),
                    *account.index(),
                    recorder.address_index,
                    recorder.address_internal,
                )
                .await?;
                unlock_blocks.push(UnlockBlock::Signature(signature.into()));
                signature_indexes.insert(recorder.address_index, current_block_index);

                // Update current block index
                current_block_index += 1;
            }
        }
        Ok(unlock_blocks)
    }
}
