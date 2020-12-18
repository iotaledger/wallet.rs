// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::StorageAdapter;
use crate::account::AccountIdentifier;

use std::path::{Path, PathBuf};

use stronghold::{RecordHint, RecordId, Stronghold};

static ACCOUNT_ID_INDEX_HINT: &str = "wallet.rs-account-ids";

type AccountIdIndex = Vec<(AccountIdentifier, RecordId)>;

/// Stronghold storage adapter.
pub struct StrongholdStorageAdapter {
    path: PathBuf,
}

impl StrongholdStorageAdapter {
    /// Initialises the storage adapter.
    pub fn new<P: AsRef<Path>>(path: P) -> crate::Result<Self> {
        Ok(Self {
            path: path.as_ref().to_path_buf(),
        })
    }
}

fn get_account_index(stronghold: &Stronghold) -> crate::Result<(RecordId, AccountIdIndex)> {
    let storage_index = stronghold.record_list()?;
    let index_hint = RecordHint::new(ACCOUNT_ID_INDEX_HINT)?;
    let (record_id, index): (RecordId, AccountIdIndex) = storage_index
        .iter()
        .find(|(record_id, record_hint)| record_hint == &index_hint)
        .map(|(record_id, record_hint)| {
            let index_json = stronghold
                .record_read(record_id)
                .expect("failed to read account id index");
            let index: AccountIdIndex = serde_json::from_str(&index_json).expect("cannot decode account id index");
            (*record_id, index)
        })
        .unwrap_or_else(|| {
            let index = AccountIdIndex::default();
            let record_id = stronghold
                .record_create(&serde_json::to_string(&index).expect("failed to encode account id index"))
                .expect("failed to save account id index");
            (record_id, index)
        });
    Ok((record_id, index))
}

fn get_from_index(
    #[allow(clippy::ptr_arg)] index: &AccountIdIndex,
    account_id: &AccountIdentifier,
) -> Option<RecordId> {
    match account_id {
        AccountIdentifier::Id(id) => index
            .iter()
            .find(|(acc_id, _)| acc_id == account_id)
            .map(|(_, record_id)| *record_id),
        AccountIdentifier::Index(pos) => {
            let pos = *pos as usize;
            if index.len() > pos {
                Some(index[pos].1)
            } else {
                None
            }
        }
    }
}

impl StorageAdapter for StrongholdStorageAdapter {
    fn get(&self, account_id: &AccountIdentifier) -> crate::Result<String> {
        let account = crate::with_stronghold_from_path(&self.path, |stronghold| {
            let (_, index) = get_account_index(&stronghold)?;
            let stronghold_id = get_from_index(&index, &account_id).ok_or(crate::WalletError::AccountNotFound)?;
            stronghold
                .record_read(&stronghold_id)
                .map_err(crate::WalletError::GenericError)
        })?;
        Ok(account)
    }

    fn get_all(&self) -> crate::Result<std::vec::Vec<String>> {
        let mut accounts = vec![];
        let (_, index) = crate::with_stronghold_from_path(&self.path, |stronghold| get_account_index(&stronghold))?;
        for (_, record_id) in index {
            let account = crate::with_stronghold_from_path(&self.path, |stronghold| {
                stronghold.record_read(&record_id).map_err(Into::into)
            })?;
            accounts.push(account);
        }
        Ok(accounts)
    }

    fn set(&self, account_id: &AccountIdentifier, account: String) -> crate::Result<()> {
        let res: crate::Result<()> = crate::with_stronghold_from_path(&self.path, |stronghold| {
            let (index_record_id, mut index) = get_account_index(&stronghold)?;
            let account_in_index = get_from_index(&index, &account_id);

            if let Some(stronghold_id) = account_in_index {
                stronghold.record_remove(stronghold_id)?;
            }

            let stronghold_id = stronghold.record_create(account.as_str())?;

            if account_in_index.is_some() {
                // account already existed; update the RecordId
                let pos = index.iter().position(|(acc_id, _)| acc_id == account_id).unwrap();
                index[pos] = (account_id.clone(), stronghold_id);
            } else {
                // new account; push to the index
                index.push((account_id.clone(), stronghold_id))
            }

            stronghold.record_remove(index_record_id)?;
            stronghold.record_create_with_hint(
                &serde_json::to_string(&index)?,
                RecordHint::new(ACCOUNT_ID_INDEX_HINT).unwrap(),
            )?;
            Ok(())
        });
        res?;

        Ok(())
    }

    fn remove(&self, account_id: &AccountIdentifier) -> crate::Result<()> {
        let res: crate::Result<()> = crate::with_stronghold_from_path(&self.path, |stronghold| {
            let (index_record_id, index) = get_account_index(&stronghold)?;
            let stronghold_id = get_from_index(&index, &account_id).ok_or(crate::WalletError::AccountNotFound)?;

            stronghold.record_remove(stronghold_id)?;

            let mut new_index = vec![];
            for (acc_id, record_id) in index {
                if &acc_id != account_id {
                    new_index.push((acc_id, record_id));
                }
            }

            stronghold.record_remove(index_record_id)?;
            stronghold
                .record_create_with_hint(
                    &serde_json::to_string(&new_index)?,
                    RecordHint::new(ACCOUNT_ID_INDEX_HINT).unwrap(),
                )
                .map_err(crate::WalletError::GenericError)?;
            Ok(())
        });
        res?;

        Ok(())
    }
}
