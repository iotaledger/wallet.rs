// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::JsAccount;
use std::{num::NonZeroU64, path::PathBuf, sync::Arc};

use iota_wallet::{
    account::AccountIdentifier,
    account_manager::{AccountManager, ManagerStorage, DEFAULT_STORAGE_FOLDER},
    client::ClientOptions,
    signing::SignerType,
    DateTime, Local,
};
use neon::prelude::*;
use serde::Deserialize;
use serde_repr::Deserialize_repr;
use tokio::sync::RwLock;

mod internal_transfer;
mod is_latest_address_unused;
mod sync;

#[derive(Deserialize_repr)]
#[repr(u8)]
pub enum AccountSignerType {
    Stronghold = 1,
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
    pub alias: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: Option<DateTime<Local>>,
    #[serde(rename = "signerType", default)]
    pub signer_type: AccountSignerType,
    #[serde(rename = "skipPersistance", default)]
    pub skip_persistance: bool,
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
            Ok((index.value() as usize).into())
        }
    }
}

pub struct AccountManagerWrapper(Arc<RwLock<AccountManager>>);

fn default_storage_path() -> PathBuf {
    DEFAULT_STORAGE_FOLDER.into()
}

#[derive(Default, Deserialize)]
struct ManagerOptions {
    #[serde(rename = "storagePath", default = "default_storage_path")]
    storage_path: PathBuf,
    #[serde(rename = "storageType")]
    storage_type: Option<ManagerStorage>,
    #[serde(rename = "storagePassword")]
    storage_password: Option<String>,
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
            let manager = crate::block_on(
                AccountManager::builder()
                .with_storage(
                    &options.storage_path,
                    options.storage_type.unwrap_or(ManagerStorage::Stronghold),
                    options.storage_password.as_deref(),
                )
                .expect("failed to init storage")
                .finish()
            ).expect("error initializing account manager");
            Ok(AccountManagerWrapper(Arc::new(RwLock::new(manager))))
        }

        method stopBackgroundSync(mut cx) {
            {
                let this = cx.this();
                let guard = cx.lock();
                let ref_ = &this.borrow(&guard).0;
                let mut manager = crate::block_on(ref_.write());
                manager.stop_background_sync()
            }
            Ok(cx.undefined().upcast())
        }

        method setStoragePassword(mut cx) {
            let password = cx.argument::<JsString>(0)?.value();
            {
                let this = cx.this();
                let guard = cx.lock();
                let ref_ = &this.borrow(&guard).0;
                crate::block_on(async move {
                    let mut manager = ref_.write().await;
                    manager.set_storage_password(password).await
                }).expect("error setting storage password");
            }
            Ok(cx.undefined().upcast())
        }

        method setStrongholdPassword(mut cx) {
            let password = cx.argument::<JsString>(0)?.value();
            {
                let this = cx.this();
                let guard = cx.lock();
                let ref_ = &this.borrow(&guard).0;
                crate::block_on(async move {
                    let mut manager = ref_.write().await;
                    manager.set_stronghold_password(password).await
                }).expect("error setting stronghold password");
            }
            Ok(cx.undefined().upcast())
        }

        method changeStrongholdPassword(mut cx) {
            let current_password = cx.argument::<JsString>(0)?.value();
            let new_password = cx.argument::<JsString>(1)?.value();
            {
                let this = cx.this();
                let guard = cx.lock();
                let ref_ = &this.borrow(&guard).0;
                crate::block_on(async move {
                    let manager = ref_.write().await;
                    manager.change_stronghold_password(current_password, new_password).await
                }).expect("error changing stronghold password");
            }
            Ok(cx.undefined().upcast())
        }

        method generateMnemonic(mut cx) {
            let mnemonic = {
                let this = cx.this();
                let guard = cx.lock();
                let ref_ = &this.borrow(&guard).0;
                let mut manager = crate::block_on(ref_.write());
                manager.generate_mnemonic().expect("failed to generate mnemonic")
            };
            Ok(cx.string(&mnemonic).upcast())
        }

        method storeMnemonic(mut cx) {
            let signer_type = cx.argument::<JsNumber>(0)?.value() as usize;
            let signer_type: AccountSignerType = serde_json::from_str(&signer_type.to_string()).expect("invalid signer type");
            let signer_type = match signer_type {
                AccountSignerType::Stronghold => SignerType::Stronghold,
            };
            let mnemonic = match cx.argument_opt(1) {
                Some(arg) => Some(arg.downcast::<JsString>().or_throw(&mut cx)?.value()),
                None => None,
            };

            {
                let this = cx.this();
                let guard = cx.lock();
                let ref_ = &this.borrow(&guard).0;
                crate::block_on(async move {
                    let mut manager = ref_.write().await;
                    manager.store_mnemonic(signer_type, mnemonic).await
                }).expect("failed to store mnemonic");
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
                let manager = crate::block_on(ref_.read());

                let mut builder = manager
                    .create_account(account_to_create.client_options)
                    .expect("failed to create account")
                    .signer_type(match account_to_create.signer_type {
                        AccountSignerType::Stronghold => SignerType::Stronghold,
                    });
                if let Some(alias) = &account_to_create.alias {
                    builder = builder.alias(alias);
                }
                if let Some(created_at) = &account_to_create.created_at {
                    builder = builder.created_at(*created_at);
                }
                if account_to_create.skip_persistance {
                    builder = builder.skip_persistance();
                }

                crate::block_on(builder.initialise()).expect("error creating account")
            };

            let id = crate::block_on(crate::store_account(account));
            let id = cx.string(serde_json::to_string(&id).unwrap());

            Ok(JsAccount::new(&mut cx, vec![id])?.upcast())
        }

        method getAccount(mut cx) {
            let id = cx.argument::<JsValue>(0)?;
            let id = js_value_to_account_id(&mut cx, id)?;
            let account = {
                let this = cx.this();
                let guard = cx.lock();
                let ref_ = &this.borrow(&guard).0;
                crate::block_on(async move {
                    let manager = ref_.read().await;
                    manager.get_account(id).await
                })
            };
            match account {
                Ok(account) => {
                    let id = crate::block_on(crate::store_account(account));
                    let id = cx.string(serde_json::to_string(&id).unwrap());
                    Ok(JsAccount::new(&mut cx, vec![id])?.upcast())
                },
                Err(_) => Ok(cx.undefined().upcast())
            }
        }

        method getAccounts(mut cx) {
            let accounts = {
                let this = cx.this();
                let guard = cx.lock();
                let ref_ = &this.borrow(&guard).0;
                crate::block_on(async move {
                    let manager = ref_.read().await;
                    manager.get_accounts().await.unwrap()
                })
            };

            let js_array = JsArray::new(&mut cx, accounts.len() as u32);
            for (index, account) in accounts.into_iter().enumerate() {
                let id = crate::block_on(crate::store_account(account));
                let id = cx.string(serde_json::to_string(&id).unwrap());
                let js_account = JsAccount::new(&mut cx, vec![id])?;
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
                crate::block_on(async move {
                    let manager = ref_.read().await;
                    manager.remove_account(id).await
                }).expect("error removing account")
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
                amount: NonZeroU64::new(amount).expect("amount can't be zero"),
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
                crate::block_on(async move {
                    let manager = ref_.read().await;
                    manager.backup(backup_path).await
                }).expect("error performing backup").display().to_string()
            };
            Ok(cx.string(destination).upcast())
        }

        method importAccounts(mut cx) {
            let source = cx.argument::<JsString>(0)?.value();
            let password = cx.argument::<JsString>(1)?.value();
            {
                let this = cx.this();
                let guard = cx.lock();
                let ref_ = &this.borrow(&guard).0;
                crate::block_on(async move {
                    let mut manager = ref_.write().await;
                    manager.import_accounts(source, password).await
                }).expect("error importing accounts");
            };
            Ok(cx.undefined().upcast())
        }

        method isLatestAddressUnused(mut cx) {
            let cb = cx.argument::<JsFunction>(0)?;

            let this = cx.this();
            let manager = cx.borrow(&this, |r| r.0.clone());
            let task = is_latest_address_unused::IsLatestAddressUnusedTask {
                manager,
            };
            task.schedule(cb);
            Ok(cx.undefined().upcast())
        }
    }
}
