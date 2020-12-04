//! Stronghold interface abstractions over an account

use crate::account::{Account, AccountIdentifier};
use futures::future::RemoteHandle;
use iota_stronghold::{ClientMsg, RecordHint, RecordId, SHRequest, SHResults, VaultId};
use once_cell::sync::{Lazy, OnceCell};
use riker::actors::*;
use riker_patterns::ask::ask;

use std::{
    collections::HashMap,
    convert::TryInto,
    fmt::{Display, Formatter, Result as FmtResult},
    path::{Path, PathBuf},
    sync::{
        mpsc::{
            channel as mpsc_channel, Receiver as MpscReceiver, RecvTimeoutError,
            Sender as MpscSender,
        },
        Arc, Mutex, RwLock,
    },
    time::Duration,
};

static PASSWORD_STORE: OnceCell<Arc<Mutex<HashMap<PathBuf, String>>>> = OnceCell::new();
static CURRENT_SNAPSHOT_PATH: OnceCell<Arc<RwLock<Option<PathBuf>>>> = OnceCell::new();

const SEED_HINT: &str = "IOTA_WALLET_SEED";
const ACCOUNT_HINT: &str = "IOTA_WALLET_ACCOUNT";
const TIMEOUT: Duration = Duration::from_millis(5000);

macro_rules! wait_for_result {
    ($self:ident, $a:pat, $b:block) => {{
        let result_rx = $self.result_rx.lock().unwrap();
        let result = result_rx.recv_timeout(TIMEOUT)?;
        std::mem::drop(result_rx);
        if let $a = result {
            $b
        } else {
            return Err(Error::UnexpectedResult(result));
        }
    }};
    ($self:ident, $a:pat, $b:block, $r:expr) => {{
        let result_rx = $self.result_rx.lock().unwrap();
        let result = result_rx.recv_timeout(TIMEOUT)?;
        std::mem::drop(result_rx);
        if let $a = result {
            $b
        } else {
            return Err($r);
        }
    }};
}

macro_rules! send_message {
    ($snapshot_path:ident, $message:expr, $expected_response:pat, $b:block) => {{
        let runtime = actor_runtime().lock().unwrap();
        check_snapshot(&runtime, $snapshot_path).await?;

        let handle: StrongholdRemoteHandle =
            ask(&runtime.system, &runtime.stronghold_actor, $message);
        let result = handle.await.map_err(|e| Error::FailedToPerformAction(e))?;
        if let $expected_response = result {
            $b
        } else {
            return Err(Error::FailedToPerformAction(format!("{:?}", result)));
        }
    }};
}

#[derive(Default)]
pub struct StrongholdSigner;

impl crate::signing::Signer for StrongholdSigner {
    /// Initialises an account.
    fn init_account(&self, account: &Account, mnemonic: Option<String>) -> crate::Result<String> {
        unimplemented!()
    }

    /// Generates an address.
    fn generate_address(
        &self,
        account: &Account,
        index: usize,
        internal: bool,
    ) -> crate::Result<iota::Ed25519Address> {
        unimplemented!()
    }

    /// Sign message.
    fn sign_message(
        &self,
        account: &Account,
        essence: &iota::TransactionEssence,
        inputs: &mut Vec<crate::signing::TransactionInput>,
    ) -> crate::Result<Vec<iota::UnlockBlock>> {
        unimplemented!()
    }
}

fn set_password<S: AsRef<Path>, P: Into<String>>(snapshot_path: S, password: P) {
    let mut passwords = PASSWORD_STORE.get_or_init(Default::default).lock().unwrap();
    passwords.insert(snapshot_path.as_ref().to_path_buf(), password.into());
}

// TODO: add error handling to this fn and consumers
fn get_password<P: AsRef<Path>>(snapshot_path: P) -> Option<String> {
    let passwords = PASSWORD_STORE.get_or_init(Default::default).lock().unwrap();
    passwords
        .get(&snapshot_path.as_ref().to_path_buf())
        .cloned()
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("`{0}`")]
    Timeout(#[from] RecvTimeoutError),
    #[error("account id isn't a valid record hint")]
    InvalidAccountIdentifier,
    #[error("must provide account id instead of string")]
    AccountIdMustBeString,
    #[error("`{0}`")]
    StrongholdError(#[from] iota_stronghold::Error),
    #[error("account not found")]
    AccountNotFound,
    #[error("snapshot doesn't have accounts")]
    EmptySnapshot,
    #[error("unexpected stronghold response type: `{0}`")]
    UnexpectedResult(StrongholdResult),
    #[error("failed to perform action: `{0}`")]
    FailedToPerformAction(String),
}

pub type Result<T> = std::result::Result<T, Error>;

type StrongholdRemoteHandle = RemoteHandle<std::result::Result<StrongholdResponse, String>>;

#[derive(Debug, Clone)]
pub enum Request {
    LoadSnapshot(PathBuf, String),
    SaveSnapshot(PathBuf, String),
    CreateSnapshot(PathBuf, String),
    GetAccountRecordId,
    GetRecord(RecordType, RecordId),
    GetAccounts,
    StoreRecord(Option<RecordId>, Vec<u8>, RecordType),
    RemoveRecord(RecordId),
}

enum Crypto {
    GenAddress,
}

#[derive(Debug)]
pub enum StrongholdResult {
    InitialisedRecord(RecordId),
    ReadRecord(Vec<u8>),
    ListIds(Vec<(RecordId, RecordHint)>),
    CreatedVault(VaultId),
    ReadSnapshot(Vec<VaultId>),
    Error(String),
}

impl Display for StrongholdResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum StrongholdResponse {
    LoadedSnapshot,
    SavedSnapshot,
    InitialisedRecord(RecordId),
    Accounts(Vec<Vec<u8>>),
    Record(Vec<u8>),
    StoredRecord(RecordId),
    RemovedRecord,
    CreatedSnapshot,
}

#[actor(SHResults)]
struct StrongholdResultReceiver {
    channel: ChannelRef<SHResults>,
    result_tx: Arc<Mutex<MpscSender<StrongholdResult>>>,
}

impl
    ActorFactoryArgs<(
        ChannelRef<SHResults>,
        Arc<Mutex<MpscSender<StrongholdResult>>>,
    )> for StrongholdResultReceiver
{
    fn create_args(
        (channel, result_tx): (
            ChannelRef<SHResults>,
            Arc<Mutex<MpscSender<StrongholdResult>>>,
        ),
    ) -> Self {
        StrongholdResultReceiver { channel, result_tx }
    }
}

impl StrongholdResultReceiver {
    fn receive_response(
        &mut self,
        ctx: &Context<StrongholdResultReceiverMsg>,
        msg: SHResults,
    ) -> Result<()> {
        println!("response: {:?}", msg);
        let result_tx = self.result_tx.lock().unwrap();
        match msg {
            SHResults::ReturnRebuild(vaults, vault_records) => {
                result_tx
                    .send(StrongholdResult::ReadSnapshot(vaults))
                    .unwrap();
            }
            SHResults::ReturnList(records) => {
                result_tx.send(StrongholdResult::ListIds(records)).unwrap();
            }
            SHResults::ReturnCreate(vault_id, record_id) => {
                result_tx
                    .send(StrongholdResult::CreatedVault(vault_id))
                    .unwrap();
            }
            SHResults::ReturnInit(vault_id, record_id) => {
                result_tx
                    .send(StrongholdResult::InitialisedRecord(record_id))
                    .unwrap();
            }
            SHResults::ReturnRead(record) => {
                result_tx
                    .send(StrongholdResult::ReadRecord(record))
                    .unwrap();
            }
        }
        Ok(())
    }
}

impl Actor for StrongholdResultReceiver {
    type Msg = StrongholdResultReceiverMsg;

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

impl Receive<SHResults> for StrongholdResultReceiver {
    type Msg = StrongholdResultReceiverMsg;

    fn receive(&mut self, ctx: &Context<Self::Msg>, msg: SHResults, sender: Sender) {
        let _ = self.receive_response(ctx, msg);
    }
}

#[actor(Request)]
struct WalletStronghold {
    result_rx: Arc<Mutex<MpscReceiver<StrongholdResult>>>,
    seed_vault: Option<VaultId>,
    accounts_vault: Option<VaultId>,
}

impl ActorFactoryArgs<Arc<Mutex<MpscReceiver<StrongholdResult>>>> for WalletStronghold {
    fn create_args(result_rx: Arc<Mutex<MpscReceiver<StrongholdResult>>>) -> Self {
        WalletStronghold {
            result_rx,
            seed_vault: None,
            accounts_vault: None,
        }
    }
}

impl Actor for WalletStronghold {
    type Msg = WalletStrongholdMsg;

    fn recv(&mut self, ctx: &Context<Self::Msg>, msg: Self::Msg, sender: Sender) {
        self.receive(ctx, msg, sender);
    }
}

fn account_id_to_record_id(account_id: AccountIdentifier) -> Result<RecordId> {
    let account_id_str = match account_id {
        AccountIdentifier::Id(id) => id,
        AccountIdentifier::Index(_) => {
            return Err(Error::AccountIdMustBeString);
        }
    };
    let id: RecordId = account_id_str.as_bytes()[0..24]
        .try_into()
        .map_err(|_| Error::InvalidAccountIdentifier)?;
    Ok(id)
}

/// Stronghold record type.
/// Determines the vault where the record will be stored, and its policy.
#[derive(Debug, Clone)]
pub enum RecordType {
    /// Seed record.
    Seed,
    /// Account record.
    Account,
}

impl WalletStronghold {
    fn clear_state(&mut self) {
        self.seed_vault = None;
        self.accounts_vault = None;
    }

    fn record_type_metadata(&self, record_type: &RecordType) -> (VaultId, RecordHint) {
        match record_type {
            RecordType::Seed => (
                self.seed_vault.unwrap(),
                RecordHint::new(SEED_HINT.as_bytes()).unwrap(),
            ),
            RecordType::Account => (
                self.accounts_vault.unwrap(),
                RecordHint::new(ACCOUNT_HINT.as_bytes()).unwrap(),
            ),
        }
    }

    fn receive_message(
        &mut self,
        ctx: &Context<WalletStrongholdMsg>,
        msg: Request,
    ) -> Result<StrongholdResponse> {
        let stronghold_client = ctx
            .select("/user/stronghold-internal/")
            .expect("failed to select stronghold actor");
        match msg {
            Request::LoadSnapshot(snapshot_path, password) => {
                self.clear_state();

                // read snapshot
                stronghold_client.try_tell(
                    ClientMsg::SHRequest(SHRequest::ReadSnapshot(
                        password,
                        None,
                        Some(snapshot_path),
                    )),
                    None,
                );
                wait_for_result!(self, StrongholdResult::ReadSnapshot(vaults), {
                    // search vault with the seed and vault with the accounts
                    let mut vault_records = vec![];
                    for vault in vaults.iter() {
                        stronghold_client
                            .try_tell(ClientMsg::SHRequest(SHRequest::ListIds(*vault)), None);
                        let seed_hint = RecordHint::new(SEED_HINT).unwrap();
                        let account_hint = RecordHint::new(ACCOUNT_HINT).unwrap();
                        wait_for_result!(self, StrongholdResult::ListIds(records), {
                            if records.iter().any(|(_, hint)| hint == &seed_hint) {
                                self.seed_vault = Some(*vault);
                            }
                            if records.iter().any(|(_, hint)| hint == &account_hint) {
                                self.accounts_vault = Some(*vault);
                            }
                            vault_records.push((*vault, records));
                            if self.seed_vault.is_some() && self.accounts_vault.is_some() {
                                break;
                            }
                        });
                    }

                    if self.seed_vault.is_none() {
                        // if the last vault is empty, use it as the seed vault
                        match vault_records.last() {
                            Some((vault_id, records)) if records.is_empty() => {
                                self.seed_vault = Some(*vault_id);
                            }
                            _ => {
                                stronghold_client.try_tell(
                                    ClientMsg::SHRequest(SHRequest::CreateNewVault),
                                    None,
                                );
                                wait_for_result!(self, StrongholdResult::CreatedVault(vault_id), {
                                    self.seed_vault = Some(vault_id);
                                });
                            }
                        }
                    }
                    if self.accounts_vault.is_none() {
                        self.accounts_vault = Some(self.seed_vault.unwrap());
                    }
                    Ok(StrongholdResponse::LoadedSnapshot)
                })
            }
            Request::SaveSnapshot(snapshot_path, password) => {
                stronghold_client.try_tell(
                    ClientMsg::SHRequest(SHRequest::WriteSnapshot(
                        password,
                        None,
                        Some(snapshot_path),
                    )),
                    None,
                );
                Ok(StrongholdResponse::SavedSnapshot)
            }
            Request::CreateSnapshot(snapshot_path, password) => {
                self.clear_state();

                stronghold_client.try_tell(ClientMsg::SHRequest(SHRequest::ClearCache), None);

                stronghold_client.try_tell(ClientMsg::SHRequest(SHRequest::CreateNewVault), None);
                wait_for_result!(self, StrongholdResult::CreatedVault(vault_id), {
                    self.seed_vault = Some(vault_id);
                    self.accounts_vault = Some(vault_id);

                    stronghold_client.try_tell(
                        ClientMsg::SHRequest(SHRequest::WriteSnapshot(
                            password,
                            None,
                            Some(snapshot_path),
                        )),
                        None,
                    );

                    Ok(StrongholdResponse::CreatedSnapshot)
                })
            }
            Request::GetAccountRecordId => {
                stronghold_client.try_tell(
                    ClientMsg::SHRequest(SHRequest::InitRecord(self.accounts_vault.unwrap())),
                    None,
                );
                wait_for_result!(self, StrongholdResult::InitialisedRecord(record_id), {
                    Ok(StrongholdResponse::InitialisedRecord(record_id))
                })
            }
            Request::GetRecord(record_type, record_id) => {
                stronghold_client.try_tell(
                    ClientMsg::SHRequest(SHRequest::ReadData(
                        self.record_type_metadata(&record_type).0,
                        Some(record_id),
                    )),
                    None,
                );
                wait_for_result!(
                    self,
                    StrongholdResult::ReadRecord(record),
                    { Ok(StrongholdResponse::Record(record)) },
                    Error::AccountNotFound
                )
            }
            Request::GetAccounts => {
                let vault_id = self.accounts_vault.ok_or_else(|| Error::EmptySnapshot)?;
                stronghold_client
                    .try_tell(ClientMsg::SHRequest(SHRequest::ListIds(vault_id)), None);
                wait_for_result!(self, StrongholdResult::ListIds(record_pairs), {
                    let mut accounts = vec![];
                    let account_hint = RecordHint::new(ACCOUNT_HINT).unwrap();
                    for (id, hint) in record_pairs {
                        if hint == account_hint {
                            stronghold_client.try_tell(
                                ClientMsg::SHRequest(SHRequest::ReadData(
                                    self.accounts_vault.unwrap(),
                                    Some(id),
                                )),
                                None,
                            );
                            wait_for_result!(
                                self,
                                StrongholdResult::ReadRecord(record),
                                {
                                    accounts.push(record);
                                },
                                Error::AccountNotFound
                            );
                        }
                    }
                    Ok(StrongholdResponse::Accounts(accounts))
                })
            }
            Request::StoreRecord(record_id, record, record_type) => {
                let (vault_id, record_hint) = self.record_type_metadata(&record_type);
                stronghold_client.try_tell(
                    ClientMsg::SHRequest(SHRequest::WriteData(
                        vault_id,
                        record_id,
                        record,
                        record_hint,
                    )),
                    None,
                );
                // TODO wait for record id response
                Ok(StrongholdResponse::StoredRecord(record_id.unwrap()))
            }
            Request::RemoveRecord(record_id) => {
                stronghold_client.try_tell(
                    ClientMsg::SHRequest(SHRequest::RevokeData(
                        self.accounts_vault.unwrap(),
                        record_id,
                    )),
                    None,
                );
                Ok(StrongholdResponse::RemovedRecord)
            }
        }
    }
}

impl Receive<Request> for WalletStronghold {
    type Msg = WalletStrongholdMsg;

    fn receive(&mut self, ctx: &Context<Self::Msg>, msg: Request, sender: Sender) {
        let res = self.receive_message(ctx, msg);
        sender
            .as_ref()
            .unwrap()
            .try_tell(res.map_err(|e| e.to_string()), Some(ctx.myself().into()))
            .unwrap();
    }
}

struct ActorRuntime {
    system: ActorSystem,
    stronghold_channel: ChannelRef<SHResults>,
    stronghold_actor: ActorRef<WalletStrongholdMsg>,
}

fn actor_runtime() -> &'static Arc<Mutex<ActorRuntime>> {
    static SYSTEM: Lazy<Arc<Mutex<ActorRuntime>>> = Lazy::new(|| {
        let system = ActorSystem::new().unwrap();
        let (system, stronghold_channel) = iota_stronghold::init_stronghold(system);
        let (result_tx, result_rx) = mpsc_channel();
        let stronghold_result_receiver_actor = system
            .actor_of_args::<StrongholdResultReceiver, _>(
                "wallet-stronghold-result-receiver",
                (stronghold_channel.clone(), Arc::new(Mutex::new(result_tx))),
            )
            .expect("failed to initialise stronghold actor");
        let stronghold_actor = system
            .actor_of_args::<WalletStronghold, _>(
                "wallet-stronghold",
                Arc::new(Mutex::new(result_rx)),
            )
            .expect("failed to initialise stronghold actor");
        let runtime = ActorRuntime {
            system,
            stronghold_channel,
            stronghold_actor,
        };
        Arc::new(Mutex::new(runtime))
    });
    &SYSTEM
}

// check if the snapshot path is different than the current loaded one
// if it is, write the current snapshot and load the new one
async fn check_snapshot(runtime: &ActorRuntime, snapshot_path: &PathBuf) -> Result<()> {
    let current_snapshot_path = CURRENT_SNAPSHOT_PATH
        .get_or_init(Default::default)
        .read()
        .unwrap();
    let curr_snapshot_path = current_snapshot_path.clone();
    std::mem::drop(current_snapshot_path);

    if let Some(curr_snapshot_path) = &curr_snapshot_path {
        // if the current loaded snapshot is different than the snapshot we're tring to use,
        // save the current snapshot and read the new snapshot
        if curr_snapshot_path != snapshot_path {
            println!("curr: {:?}, next: {:?}", curr_snapshot_path, &snapshot_path,);
            let curr_snapshot_password = get_password(&curr_snapshot_path).unwrap();
            let handle: StrongholdRemoteHandle = ask(
                &runtime.system,
                &runtime.stronghold_actor,
                Request::SaveSnapshot(curr_snapshot_path.to_path_buf(), curr_snapshot_password),
            );

            let result = handle.await.map_err(|e| Error::FailedToPerformAction(e))?;
            if let StrongholdResponse::SavedSnapshot = result {
                let password = get_password(snapshot_path).unwrap();
                println!(
                    "next: {:?}, exists: {}",
                    snapshot_path,
                    snapshot_path.exists()
                );

                let (handle, expected_response): (StrongholdRemoteHandle, StrongholdResponse) =
                    if snapshot_path.exists() {
                        (
                            ask(
                                &runtime.system,
                                &runtime.stronghold_actor,
                                Request::LoadSnapshot(snapshot_path.to_path_buf(), password),
                            ),
                            StrongholdResponse::LoadedSnapshot,
                        )
                    } else {
                        (
                            ask(
                                &runtime.system,
                                &runtime.stronghold_actor,
                                Request::CreateSnapshot(snapshot_path.to_path_buf(), password),
                            ),
                            StrongholdResponse::CreatedSnapshot,
                        )
                    };
                let result = handle.await.map_err(|e| Error::FailedToPerformAction(e))?;
                if expected_response == result {
                    let mut current_snapshot_path = CURRENT_SNAPSHOT_PATH
                        .get_or_init(Default::default)
                        .write()
                        .unwrap();
                    current_snapshot_path.replace(snapshot_path.clone());
                } else {
                    return Err(Error::FailedToPerformAction(format!("{:?}", result)));
                }
            } else {
                return Err(Error::FailedToPerformAction(format!("{:?}", result)));
            }
        }
    }

    Ok(())
}

pub async fn load_or_create<P: Into<String>>(snapshot_path: &PathBuf, password: P) -> Result<()> {
    let password = password.into();
    set_password(snapshot_path, password.clone());
    let res: Result<()> = if snapshot_path.exists() {
        send_message!(
            snapshot_path,
            Request::LoadSnapshot(snapshot_path.clone(), password),
            StrongholdResponse::LoadedSnapshot,
            { Ok(()) }
        )
    } else {
        send_message!(
            snapshot_path,
            Request::CreateSnapshot(snapshot_path.clone(), password),
            StrongholdResponse::CreatedSnapshot,
            { Ok(()) }
        )
    };
    res?;

    let mut current_snapshot_path = CURRENT_SNAPSHOT_PATH
        .get_or_init(Default::default)
        .write()
        .unwrap();
    current_snapshot_path.replace(snapshot_path.clone());

    Ok(())
}

pub async fn do_crypto(account: &Account) -> Result<()> {
    Ok(())
}

pub async fn get_accounts(snapshot_path: &PathBuf) -> Result<Vec<String>> {
    send_message!(
        snapshot_path,
        Request::GetAccounts,
        StrongholdResponse::Accounts(accounts),
        {
            Ok(accounts
                .into_iter()
                .map(|acc| String::from_utf8_lossy(&acc).to_string())
                .collect())
        }
    )
}

pub async fn read_record(
    snapshot_path: &PathBuf,
    record_type: RecordType,
    record_id: RecordId,
) -> Result<Vec<u8>> {
    send_message!(
        snapshot_path,
        Request::GetRecord(record_type, record_id),
        StrongholdResponse::Record(record),
        { Ok(record) }
    )
}

pub async fn get_account(snapshot_path: &PathBuf, account_id: AccountIdentifier) -> Result<String> {
    let raw = read_record(
        snapshot_path,
        RecordType::Account,
        account_id_to_record_id(account_id)?,
    )
    .await?;
    Ok(String::from_utf8_lossy(&raw).to_string())
}

pub async fn new_account_record(snapshot_path: &PathBuf) -> Result<RecordId> {
    send_message!(
        snapshot_path,
        Request::GetAccountRecordId,
        StrongholdResponse::InitialisedRecord(record_id),
        { Ok(record_id) }
    )
}

pub async fn store_record(
    snapshot_path: &PathBuf,
    record_id: Option<RecordId>,
    record: Vec<u8>,
    record_type: RecordType,
) -> Result<RecordId> {
    send_message!(
        snapshot_path,
        Request::StoreRecord(record_id, record, record_type),
        StrongholdResponse::StoredRecord(id),
        { Ok(id) }
    )
}

pub async fn store_account(
    snapshot_path: &PathBuf,
    account_id: AccountIdentifier,
    account: String,
) -> Result<()> {
    store_record(
        snapshot_path,
        Some(account_id_to_record_id(account_id)?),
        account.as_bytes().to_vec(),
        RecordType::Account,
    )
    .await?;
    Ok(())
}

pub async fn remove_record(snapshot_path: &PathBuf, record_id: RecordId) -> Result<()> {
    send_message!(
        snapshot_path,
        Request::RemoveRecord(record_id),
        StrongholdResponse::RemovedRecord,
        { Ok(()) }
    )
}

pub async fn remove_account(snapshot_path: &PathBuf, account_id: AccountIdentifier) -> Result<()> {
    remove_record(snapshot_path, account_id_to_record_id(account_id)?).await
}

#[cfg(test)]
mod tests {
    use super::RecordType;
    use rand::{thread_rng, Rng};
    use std::path::PathBuf;

    #[tokio::test]
    async fn write_and_read() -> super::Result<()> {
        let snapshot_path: String = thread_rng().gen_ascii_chars().take(10).collect();
        let snapshot_path = PathBuf::from(format!("./example-database/{}", snapshot_path));
        println!("using {:?}", snapshot_path);
        super::load_or_create(&snapshot_path, "password").await?;

        let data = "account data";
        let record_id = super::new_account_record(&snapshot_path).await?;
        super::store_record(
            &snapshot_path,
            Some(record_id),
            data.clone().as_bytes().to_vec(),
            RecordType::Account,
        )
        .await?;
        let stored_data =
            super::read_record(&snapshot_path, RecordType::Account, record_id).await?;
        assert_eq!(stored_data, data.as_bytes().to_vec());

        Ok(())
    }

    #[tokio::test]
    async fn write_and_read_multiple_snapshots() -> super::Result<()> {
        let mut snapshot_saves = vec![];
        let _ = std::fs::create_dir("./example-database");

        for _ in 1..3 {
            let snapshot_path: String = thread_rng().gen_ascii_chars().take(10).collect();
            let snapshot_path = PathBuf::from(format!("./example-database/{}", snapshot_path));
            println!("using {:?}", snapshot_path);
            super::load_or_create(&snapshot_path, "password").await?;

            let data: String = thread_rng().gen_ascii_chars().take(10).collect();
            let record_id = super::new_account_record(&snapshot_path).await?;
            super::store_record(
                &snapshot_path,
                Some(record_id),
                data.clone().as_bytes().to_vec(),
                RecordType::Account,
            )
            .await?;
            snapshot_saves.push((snapshot_path, record_id, data));
            println!("\n\n\n\n");
        }

        for (snapshot_path, record_id, data) in snapshot_saves {
            let stored_data =
                super::read_record(&snapshot_path, RecordType::Account, record_id).await?;
            assert_eq!(stored_data, data.as_bytes().to_vec());
        }

        Ok(())
    }
}
