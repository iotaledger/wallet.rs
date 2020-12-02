//! Stronghold interface abstractions over an account

use crate::account::{Account, AccountIdentifier};
use iota_stronghold::{RecordHint, RecordId, SHRequest, SHResults, VaultId};
use once_cell::sync::{Lazy, OnceCell};
use riker::actors::*;

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

static PASSWORD_STORE: OnceCell<Arc<Mutex<HashMap<PathBuf, String>>>> = OnceCell::new();

static SEED_HINT: &str = "IOTA_WALLET_SEED";
const ACCOUNT_HINT: &str = "IOTA_WALLET_ACCOUNT";

fn set_password<S: AsRef<Path>, P: Into<String>>(snapshot_path: S, password: P) {
    let mut passwords = PASSWORD_STORE.get_or_init(Default::default).lock().unwrap();
    passwords.insert(snapshot_path.as_ref().to_path_buf(), password.into());
}

fn get_password<P: AsRef<Path>>(snapshot_path: P) -> Option<String> {
    let passwords = PASSWORD_STORE.get_or_init(Default::default).lock().unwrap();
    passwords
        .get(&snapshot_path.as_ref().to_path_buf())
        .cloned()
}

#[derive(Debug, Clone)]
pub enum Request {
    LoadSnapshot(PathBuf, String),
    GetAccount(AccountIdentifier),
    GetAccounts,
    StoreAccount(Account),
    RemoveAccount(AccountIdentifier),
}

enum Crypto {
    GenAddress,
}

#[derive(Default)]
struct WalletStrongholdInnerState {
    list_vault_id: Option<VaultId>,
    read_record_id: Option<RecordId>,
    created_vault_id: Option<VaultId>,
}

#[actor(SHResults, Request)]
struct WalletStronghold {
    channel: ChannelRef<SHResults>,
    vaults: Vec<VaultId>,
    seed_vault: Option<VaultId>,
    accounts_vault: Option<VaultId>,
    record_ids: Vec<(RecordId, RecordHint)>,
    records: HashMap<RecordId, Vec<u8>>,
    inner_state: WalletStrongholdInnerState,
}

impl ActorFactoryArgs<ChannelRef<SHResults>> for WalletStronghold {
    fn create_args(channel: ChannelRef<SHResults>) -> Self {
        WalletStronghold {
            channel,
            vaults: Default::default(),
            seed_vault: None,
            accounts_vault: None,
            record_ids: vec![],
            records: Default::default(),
            inner_state: Default::default(),
        }
    }
}

impl Actor for WalletStronghold {
    type Msg = WalletStrongholdMsg;

    // set up the channel.
    fn pre_start(&mut self, ctx: &Context<Self::Msg>) {
        let sub = Box::new(ctx.myself());
        let topic = Topic::from("external");
        self.channel.tell(Subscribe { actor: sub, topic }, None);
    }

    fn recv(&mut self, ctx: &Context<Self::Msg>, msg: Self::Msg, sender: Sender) {
        self.receive(ctx, msg, sender);
    }
}

fn account_id_to_record_hint(account_id: AccountIdentifier) -> crate::Result<RecordHint> {
    let account_id_str = match account_id {
        AccountIdentifier::Id(id) => id,
        AccountIdentifier::Index(_) => {
            panic!("must provide account id instead of index")
        }
    };
    let hint = RecordHint::new(account_id_str.as_bytes()).map_err(|e| {
        anyhow::anyhow!("account id isn't a valid record hint: {:?}", e.to_string())
    })?;
    Ok(hint)
}

impl WalletStronghold {
    fn receive_message(
        &mut self,
        ctx: &Context<WalletStrongholdMsg>,
        msg: Request,
        _sender: Sender,
    ) -> crate::Result<()> {
        let stronghold_client = ctx
            .select("/user/stronghold-internal")
            .expect("failed to select stronghold actor");
        match msg {
            Request::LoadSnapshot(snapshot_path, password) => {
                // clear actor state
                self.vaults = vec![];
                self.seed_vault = None;
                self.accounts_vault = None;
                self.records = Default::default();
                self.record_ids = vec![];
                self.inner_state = Default::default();

                // refresh password
                set_password(snapshot_path, password);

                // read snapshot
                stronghold_client.try_tell(
                    SHRequest::ReadSnapshot(password, None, Some(snapshot_path)),
                    None,
                );
            }
            Request::GetAccount(account_id) => {
                let account_id_hint = account_id_to_record_hint(account_id)?;
                let record_id = self
                    .record_ids
                    .iter()
                    .find(|(_, hint)| hint == &account_id_hint)
                    .ok_or_else(|| crate::WalletError::AccountNotFound)?
                    .0;
                stronghold_client.try_tell(
                    SHRequest::ReadData(self.accounts_vault.unwrap(), Some(record_id.clone())),
                    None,
                );
            }
            Request::GetAccounts => {
                let vault_id = self
                    .accounts_vault
                    .ok_or_else(|| anyhow::anyhow!("snapshot doesn't have accounts"))?;
            }
            Request::StoreAccount(account) => stronghold_client.try_tell(
                SHRequest::WriteData(
                    self.accounts_vault.unwrap(),
                    None,
                    serde_json::to_string(&account)?.as_bytes().to_vec(),
                    RecordHint::new(account.id().as_bytes()).map_err(|e| {
                        anyhow::anyhow!("account id isn't a valid record hint: {:?}", e.to_string())
                    })?,
                ),
                None,
            ),
            Request::RemoveAccount(account_id) => {
                let account_id_hint = account_id_to_record_hint(account_id)?;
                let record_id = self
                    .record_ids
                    .iter()
                    .find(|(_, hint)| hint == &account_id_hint)
                    .ok_or_else(|| crate::WalletError::AccountNotFound)?
                    .0;
                stronghold_client.try_tell(
                    SHRequest::RevokeData(self.accounts_vault.unwrap(), record_id),
                    None,
                );
            }
        }
        Ok(())
    }
}

impl Receive<Request> for WalletStronghold {
    type Msg = WalletStrongholdMsg;

    fn receive(&mut self, ctx: &Context<Self::Msg>, msg: Request, _sender: Sender) {
        match self.receive_message(ctx, msg, _sender) {
            Ok(_) => {}
            Err(_) => {}
        }
    }
}

impl Receive<SHResults> for WalletStronghold {
    type Msg = WalletStrongholdMsg;

    fn receive(&mut self, ctx: &Context<Self::Msg>, msg: SHResults, _sender: Sender) {
        match msg {
            SHResults::ReturnRebuild(vaults, vault_records) => {
                let stronghold_client = ctx
                    .select("/user/stronghold-internal")
                    .expect("failed to select stronghold actor");
                self.vaults = vaults;
                for vault in self.vaults.iter() {
                    self.inner_state.list_vault_id = Some(*vault);
                    stronghold_client.try_tell(SHRequest::ListIds(*vault), None);
                }
                if self.seed_vault.is_none() {
                    stronghold_client.try_tell(SHRequest::CreateNewVault, None);
                    self.seed_vault = self.inner_state.created_vault_id;
                }
                if self.accounts_vault.is_none() {
                    self.accounts_vault =
                        Some(self.inner_state.created_vault_id.unwrap_or_else(|| {
                            stronghold_client.try_tell(SHRequest::CreateNewVault, None);
                            self.inner_state.created_vault_id.unwrap()
                        }));
                }
            }
            SHResults::ReturnList(records) => {
                let seed_hint = RecordHint::new(SEED_HINT).unwrap();
                let account_hint = RecordHint::new(SEED_HINT).unwrap();
                if records.iter().any(|(_, hint)| hint == &seed_hint) {
                    self.seed_vault = Some(self.inner_state.list_vault_id.unwrap());
                }
                if records.iter().any(|(_, hint)| hint == &account_hint) {
                    self.accounts_vault = Some(self.inner_state.list_vault_id.unwrap());
                }
                self.record_ids.extend(records);
            }
            SHResults::ReturnCreate(vault_id, record_id) => {}
            SHResults::ReturnInit(vault_id, record_id) => {}
            SHResults::ReturnRead(record) => {
                let record_id = self.inner_state.read_record_id.unwrap();
                self.records.insert(record_id, record);
            }
        }
    }
}

struct ActorRuntime {
    system: ActorSystem,
    stronghold_channel: ChannelRef<SHResults>,
    stronghold_actor: ActorRef<WalletStrongholdMsg>,
}

fn actor_runtime() -> &'static ActorRuntime {
    static SYSTEM: Lazy<ActorRuntime> = Lazy::new(|| {
        let system = ActorSystem::new().unwrap();
        let (system, stronghold_channel) = iota_stronghold::init_stronghold(system);
        let stronghold_actor = system
            .actor_of_args::<WalletStronghold, _>("wallet-stronghold", stronghold_channel)
            .expect("failed to initialise stronghold actor");
        ActorRuntime {
            system,
            stronghold_channel,
            stronghold_actor,
        }
    });
    &SYSTEM
}

pub fn load_snapshot<S: AsRef<Path>, P: Into<String>>(
    snapshot_path: S,
    password: P,
) -> crate::Result<()> {
    let runtime = actor_runtime();

    if snapshot_path.as_ref().exists() {
        runtime.stronghold_actor.tell(
            Request::LoadSnapshot(snapshot_path.as_ref().to_path_buf(), password.into()),
            None,
        );
    }

    Ok(())
}

pub fn do_crypto(account: &Account) -> crate::Result<()> {
    Ok(())
}

pub fn get_accounts() -> crate::Result<Vec<String>> {
    let runtime = actor_runtime();
    let mut messages = vec![];

    // runtime.stronghold_actor.try_tell(SHRequest::ListIds())

    Ok(messages)
}
