// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::account::Account;

use iota::{ReferenceUnlock, UnlockBlock};

use std::{collections::HashMap, path::PathBuf};

#[derive(Default)]
pub struct StrongholdSigner;

pub(crate) async fn stronghold_path(storage_path: &PathBuf) -> crate::Result<PathBuf> {
    let storage_id = crate::storage::get(&storage_path).await?.lock().await.id();
    let path = if storage_id == crate::storage::stronghold::STORAGE_ID {
        storage_path.clone()
    } else if storage_path.is_dir() {
        storage_path.join(crate::account_manager::STRONGHOLD_FILENAME)
    } else if let Some(parent) = storage_path.parent() {
        parent.join(crate::account_manager::STRONGHOLD_FILENAME)
    } else {
        storage_path.clone()
    };
    Ok(path)
}

#[async_trait::async_trait]
impl super::Signer for StrongholdSigner {
    async fn store_mnemonic(&mut self, storage_path: &PathBuf, mnemonic: String) -> crate::Result<()> {
        crate::stronghold::store_mnemonic(&stronghold_path(storage_path).await?, mnemonic).await?;
        Ok(())
    }

    async fn generate_address(
        &mut self,
        account: &Account,
        address_index: usize,
        internal: bool,
        _: super::GenerateAddressMetadata,
    ) -> crate::Result<iota::Address> {
        let address = crate::stronghold::generate_address(
            &stronghold_path(account.storage_path()).await?,
            *account.index(),
            address_index,
            internal,
        )
        .await?;
        Ok(address)
    }

    async fn sign_message<'a>(
        &mut self,
        account: &Account,
        essence: &iota::Essence,
        inputs: &mut Vec<super::TransactionInput>,
        _: super::SignMessageMetadata<'a>,
    ) -> crate::Result<Vec<iota::UnlockBlock>> {
        let mut unlock_blocks = vec![];
        let mut signature_indexes = HashMap::<String, usize>::new();
        inputs.sort_by(|a, b| a.input.cmp(&b.input));

        for (current_block_index, recorder) in inputs.iter().enumerate() {
            let signature_index = format!("{}{}", recorder.address_index, recorder.address_internal);
            // Check if current path is same as previous path
            // If so, add a reference unlock block
            if let Some(block_index) = signature_indexes.get(&signature_index) {
                unlock_blocks.push(UnlockBlock::Reference(ReferenceUnlock::new(*block_index as u16)?));
            } else {
                // If not, we should create a signature unlock block
                let signature = crate::stronghold::sign_transaction(
                    &stronghold_path(account.storage_path()).await?,
                    &essence.hash(),
                    *account.index(),
                    recorder.address_index,
                    recorder.address_internal,
                )
                .await?;
                unlock_blocks.push(UnlockBlock::Signature(signature.into()));
                signature_indexes.insert(signature_index, current_block_index);
            }
        }
        Ok(unlock_blocks)
    }
}
