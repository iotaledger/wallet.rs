// Copyright 2020 IOTA Stiftung
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with
// the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on
// an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and limitations under the License.

use super::StorageAdapter;
use crate::account::AccountIdentifier;

use std::path::{Path, PathBuf};

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

impl StorageAdapter for StrongholdStorageAdapter {
    fn get(&self, account_id: AccountIdentifier) -> crate::Result<String> {
        crate::stronghold::get_account(&self.path, account_id);
        unimplemented!()
    }

    fn get_all(&self) -> crate::Result<std::vec::Vec<String>> {
        crate::stronghold::get_accounts(&self.path);
        unimplemented!()
    }

    fn set(&self, account_id: AccountIdentifier, account: String) -> crate::Result<()> {
        crate::stronghold::store_account(&self.path, account_id, account);

        Ok(())
    }

    fn remove(&self, account_id: AccountIdentifier) -> crate::Result<()> {
        crate::stronghold::remove_account(&self.path, account_id);

        Ok(())
    }
}
