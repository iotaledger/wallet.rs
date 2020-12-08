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

use super::JsAccount;
use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
};

use iota_wallet::{
    account::AccountIdentifier,
    account_manager::{AccountManager, DEFAULT_STORAGE_PATH},
    client::ClientOptions,
    signing::SignerType,
    storage::{sqlite::SqliteStorageAdapter, stronghold::StrongholdStorageAdapter},
    DateTime, Utc,
};
use neon::prelude::*;
use serde::Deserialize;
use serde_repr::Deserialize_repr;

mod internal_transfer;
mod sync;

#[derive(Deserialize_repr)]
#[repr(u8)]
pub enum AccountSignerType {
    Stronghold = 1,
    EnvMnemonic = 2,
}

impl Default for AccountSignerType {
    fn default() -> Self {
        Self::Stronghold
    }
}

#[derive(Deserialize)]
pub struct AccountToCreate {
    #[serde(rename = "clientOptions")]
    pub client_options: ClientOptions,
    pub mnemonic: Option<String>,
    pub alias: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: Option<String>,
    #[serde(rename = "signerType", default)]
    pub signer_type: AccountSignerType,
}

fn js_value_to_account_id(
    cx: &mut CallContext<'_, JsAccountManager>,
    value: Handle<JsValue>,
) -> NeonResult<AccountIdentifier> {
    match value.downcast::<JsString>() {
        Ok(js_string) => {
            let id = js_string.value();
            Ok(id.into())
        }
        Err(_) => {
            let index: JsNumber = *value.downcast_or_throw(cx)?;
            Ok((index.value() as u64).into())
        }
    }
}

pub struct AccountManagerWrapper(Arc<RwLock<AccountManager>>);

#[repr(u8)]
#[derive(Deserialize_repr)]
enum StorageType {
    Stronghold = 1,
    Sqlite = 2,
}

impl Default for StorageType {
    fn default() -> Self {
        Self::Stronghold
    }
}

fn default_storage_path() -> PathBuf {
    DEFAULT_STORAGE_PATH.into()
}

#[derive(Default, Deserialize)]
struct ManagerOptions {
    #[serde(rename = "storagePath", default = "default_storage_path")]
    storage_path: PathBuf,
    #[serde(default, rename = "storageType")]
    storage_type: StorageType,
}

declare_types! {
    pub class JsAccountManager for AccountManagerWrapper {
        init(mut cx) {
            let options: ManagerOptions = match cx.argument_opt(0) {
                Some(arg) => {
                    let options = arg.downcast::<JsValue>().or_throw(&mut cx)?;
                    neon_serde::from_value(&mut cx, options)?
                }
                None => Default::default(),
            };
            let manager = match options.storage_type {
                StorageType::Sqlite => AccountManager::with_storage_adapter(&options.storage_path, SqliteStorageAdapter::new(&options.storage_path, "accounts").unwrap()),
                StorageType::Stronghold => AccountManager::with_storage_adapter(&options.storage_path, StrongholdStorageAdapter::new(&options.storage_path).unwrap()),
            };
            let manager = manager.expect("error initializing account manager");
            Ok(AccountManagerWrapper(Arc::new(RwLock::new(manager))))
        }

        method setStrongholdPassword(mut cx) {
            let password = cx.argument::<JsString>(0)?.value();
            {
                let this = cx.this();
                let guard = cx.lock();
                let ref_ = &this.borrow(&guard).0;
                let mut manager = ref_.write().unwrap();
                manager.set_stronghold_password(password).expect("error setting stronghold password");
            }
            Ok(cx.undefined().upcast())
        }

        method createAccount(mut cx) {
            let account = {
                let account_to_create = cx.argument::<JsValue>(0)?;
                let account_to_create: AccountToCreate = neon_serde::from_value(&mut cx, account_to_create)?;
                let this = cx.this();
                let guard = cx.lock();
                let ref_ = &this.borrow(&guard).0;
                let manager = ref_.read().unwrap();

                let mut builder = manager
                    .create_account(account_to_create.client_options.clone())
                    .signer_type(match account_to_create.signer_type {
                        AccountSignerType::Stronghold => SignerType::Stronghold,
                        AccountSignerType::EnvMnemonic => SignerType::EnvMnemonic,
                    });
                if let Some(mnemonic) = &account_to_create.mnemonic {
                    builder = builder.mnemonic(mnemonic);
                }
                if let Some(alias) = &account_to_create.alias {
                    builder = builder.alias(alias);
                }
                if let Some(created_at) = &account_to_create.created_at {
                    builder = builder.created_at(
                        created_at
                        .parse::<DateTime<Utc>>()
                        .expect("invalid account created at format"),
                    );
                }
                builder.initialise().expect("error creating account")
            };
            let account = serde_json::to_string(&account).unwrap();
            let account = cx.string(account);

            Ok(JsAccount::new(&mut cx, vec![account])?.upcast())
        }

        method getAccount(mut cx) {
            let id = cx.argument::<JsValue>(0)?;
            let id = js_value_to_account_id(&mut cx, id)?;
            let account = {
                let this = cx.this();
                let guard = cx.lock();
                let ref_ = &this.borrow(&guard).0;
                let manager = ref_.read().unwrap();
                manager.get_account(id)
            };
            match account {
                Ok(acc) => {
                    let account = serde_json::to_string(&acc).unwrap();
                    let account = cx.string(account);
                    Ok(JsAccount::new(&mut cx, vec![account])?.upcast())
                },
                Err(_) => Ok(cx.undefined().upcast())
            }
        }

        method getAccountByAlias(mut cx) {
            let alias = cx.argument::<JsString>(0)?.value();
            let account = {
                let this = cx.this();
                let guard = cx.lock();
                let ref_ = &this.borrow(&guard).0;
                let manager = ref_.read().unwrap();
                manager.get_account_by_alias(alias)
            };
            match account {
                Some(acc) => {
                    let account = serde_json::to_string(&acc).unwrap();
                    let account = cx.string(account);
                    Ok(JsAccount::new(&mut cx, vec![account])?.upcast())
                },
                None => Ok(cx.undefined().upcast())
            }
        }

        method getAccounts(mut cx) {
            let accounts = {
                let this = cx.this();
                let guard = cx.lock();
                let ref_ = &this.borrow(&guard).0;
                let manager = ref_.read().unwrap();
                manager.get_accounts().expect("failed to get accounts")
            };

            let js_array = JsArray::new(&mut cx, accounts.len() as u32);
            for (index, account) in accounts.iter().enumerate() {
                let account = serde_json::to_string(&account).unwrap();
                let account = cx.string(account);
                let js_account = JsAccount::new(&mut cx, vec![account])?;
                js_array.set(&mut cx, index as u32, js_account)?;
            }

            Ok(js_array.upcast())
        }

        method removeAccount(mut cx) {
            let id = cx.argument::<JsValue>(0)?;
            let id = js_value_to_account_id(&mut cx, id)?;
            {
                let this = cx.this();
                let guard = cx.lock();
                let ref_ = &this.borrow(&guard).0;
                let manager = ref_.read().unwrap();
                manager.remove_account(id).expect("error removing account")
            };
            Ok(cx.undefined().upcast())
        }

        method syncAccounts(mut cx) {
            let cb = cx.argument::<JsFunction>(0)?;
            let this = cx.this();
            let manager = cx.borrow(&this, |r| r.0.clone());
            let task = sync::SyncTask {
                manager,
            };
            task.schedule(cb);
            Ok(cx.undefined().upcast())
        }

        method internalTransfer(mut cx) {
            let from_account = cx.argument::<JsAccount>(0)?;
            let to_account = cx.argument::<JsAccount>(1)?;
            let amount = cx.argument::<JsNumber>(2)?.value() as u64;
            let cb = cx.argument::<JsFunction>(3)?;

            let from_account_id = {
                let guard = cx.lock();
                let id = from_account.borrow(&guard).0.clone();
                id
            };
            let to_account_id = {
                let guard = cx.lock();
                let id = to_account.borrow(&guard).0.clone();
                id
            };

            let this = cx.this();
            let manager = cx.borrow(&this, |r| r.0.clone());
            let task = internal_transfer::InternalTransferTask {
                manager,
                from_account_id,
                to_account_id,
                amount,
            };
            task.schedule(cb);
            Ok(cx.undefined().upcast())
        }

        method backup(mut cx) {
            let backup_path = cx.argument::<JsString>(0)?.value();
            let destination = {
                let this = cx.this();
                let guard = cx.lock();
                let ref_ = &this.borrow(&guard).0;
                let manager = ref_.read().unwrap();
                manager.backup(backup_path).expect("error performing backup").display().to_string()
            };
            Ok(cx.string(destination).upcast())
        }

        method importAccounts(mut cx) {
            let source = cx.argument::<JsString>(0)?.value();
            {
                let this = cx.this();
                let guard = cx.lock();
                let ref_ = &this.borrow(&guard).0;
                let manager = ref_.read().unwrap();
                manager.import_accounts(source).expect("error importing accounts");
            };
            Ok(cx.undefined().upcast())
        }
    }
}
