use crate::account::Account;
use getset::Getters;
use iota::crypto::ternary::sponge::{Kerl, Sponge};
use iota::signing::ternary::{
    wots::{WotsSecurityLevel, WotsShakePrivateKeyGeneratorBuilder},
    PrivateKey, PrivateKeyGenerator, PublicKey,
};
use iota::ternary::TritBuf;
pub use iota::transaction::bundled::Address as IotaAddress;
use iota::transaction::bundled::BundledTransactionField;

/// The address builder.
#[derive(Default)]
pub struct AddressBuilder {
    address: Option<IotaAddress>,
    balance: Option<u64>,
    key_index: Option<u64>,
}

impl AddressBuilder {
    /// Initialises a new instance of the address builder.
    pub fn new() -> AddressBuilder {
        Default::default()
    }

    /// Defines the address.
    pub fn address(mut self, address: IotaAddress) -> Self {
        self.address = Some(address);
        self
    }

    /// Sets the address balance.
    pub fn balance(mut self, balance: u64) -> Self {
        self.balance = Some(balance);
        self
    }

    /// Sets the address key index.
    pub fn key_index(mut self, key_index: u64) -> Self {
        self.key_index = Some(key_index);
        self
    }

    /// Builds the address.
    pub fn build(self) -> crate::Result<Address> {
        let iota_address = self
            .address
            .ok_or_else(|| anyhow::anyhow!("the `address` field is required"))?;
        let checksum = generate_checksum(&iota_address)?;
        let address = Address {
            address: iota_address,
            balance: self
                .balance
                .ok_or_else(|| anyhow::anyhow!("the `balance` field is required"))?,
            key_index: self
                .key_index
                .ok_or_else(|| anyhow::anyhow!("the `key_index` field is required"))?,
            checksum,
        };
        Ok(address)
    }
}

/// An address.
#[derive(Debug, Getters, Clone)]
#[getset(get = "pub")]
pub struct Address {
    /// The address.
    address: IotaAddress,
    /// The address balance.
    balance: u64,
    /// The address key index.
    key_index: u64,
    /// The address checksum.
    checksum: TritBuf,
}

impl PartialEq for Address {
    fn eq(&self, other: &Address) -> bool {
        self.address() == other.address()
    }
}

/// Gets an unused address for the given account.
pub(crate) async fn get_new_address(account: &Account) -> crate::Result<Address> {
    let client = crate::client::get_client(account.client_options());
    let (key_index, iota_address) = client
        .generate_new_address(account.seed())
        .generate()
        .await?;
    let balance = get_balance(&account, &iota_address).await?;
    let checksum = generate_checksum(&iota_address)?;
    let address = Address {
        address: iota_address,
        balance,
        key_index,
        checksum,
    };
    Ok(address)
}

/// Batch address generation.
pub(crate) async fn get_addresses(account: &Account, count: u64) -> crate::Result<Vec<Address>> {
    let mut addresses = vec![];
    let seed_trits = account.seed().as_trits();
    for i in 0..count {
        let address: IotaAddress = IotaAddress::try_from_inner(
            WotsShakePrivateKeyGeneratorBuilder::<Kerl>::default()
                .with_security_level(WotsSecurityLevel::Medium)
                .build()
                .unwrap()
                .generate_from_entropy(seed_trits)
                .unwrap()
                .generate_public_key()
                .unwrap()
                .as_trits()
                .to_owned(),
        )
        .unwrap();
        let balance = get_balance(&account, &address).await?;
        let checksum = generate_checksum(&address)?;
        addresses.push(Address {
            address,
            balance,
            key_index: i, // TODO
            checksum,
        })
    }
    Ok(addresses)
}

/// Generates a checksum for the given address
// TODO: maybe this should be part of the crypto lib
pub(crate) fn generate_checksum(address: &IotaAddress) -> crate::Result<TritBuf> {
    let mut kerl = Kerl::new();
    let mut hash = kerl
        .digest(address.to_inner())
        .map_err(|e| anyhow::anyhow!("Erro hashing the address"))?;
    let mut trits = vec![];

    for _ in 1..10 {
        if let Some(trit) = hash.pop() {
            trits.push(trit);
        } else {
            return Err(anyhow::anyhow!("Hash error"));
        }
    }

    Ok(TritBuf::from_trits(&trits[..]))
}

async fn get_balance(account: &Account, address: &IotaAddress) -> crate::Result<u64> {
    let client = crate::client::get_client(account.client_options());
    client
        .get_balances()
        .addresses(&[address.clone()])
        .send()
        .await?
        .balances
        .first()
        .copied()
        .ok_or_else(|| anyhow::anyhow!("Balances response empty"))
}

pub(crate) fn is_unspent(account: &Account, address: &IotaAddress) -> bool {
    account
        .transactions()
        .iter()
        .any(|tx| tx.value().without_denomination() < 0 && tx.address().address() == address)
}

#[cfg(test)]
mod tests {
    use super::{Address, IotaAddress};
    use crate::account::Account;
    use crate::account_manager::AccountManager;
    use crate::client::ClientOptionsBuilder;
    use crate::transaction::{Tag, Transaction, Value, ValueUnit};

    use iota::crypto::ternary::Hash;
    use iota::ternary::TryteBuf;
    use iota::transaction::bundled::BundledTransactionField;
    use rand::Rng;

    fn _create_account() -> Account {
        let manager = AccountManager::new();

        let id = rand::thread_rng()
            .gen_ascii_chars()
            .take(5)
            .collect::<String>();
        let client_options = ClientOptionsBuilder::node("https://nodes.comnet.thetangle.org")
            .unwrap()
            .build();
        let account = manager
            .create_account(client_options)
            .alias(&id)
            .id(&id)
            .mnemonic(&id)
            .initialise()
            .unwrap();

        account
    }

    fn _create_address() -> IotaAddress {
        IotaAddress::from_inner_unchecked(
            TryteBuf::try_from_str(
                "XUERGHWTYRTFUYKFKXURKHMFEVLOIFTTCNTXOGLDPCZ9CJLKHROOPGNAQYFJEPGK9OKUQROUECBAVNXRY",
            )
            .unwrap()
            .as_trits()
            .encode(),
        )
    }

    fn _generate_transaction(value: i64, address: Address) -> Transaction {
        Transaction {
            hash: Hash::zeros(),
            address,
            value: Value::new(value, ValueUnit::I),
            tag: Tag::default(),
            timestamp: chrono::Utc::now(),
            current_index: 0,
            last_index: 0,
            bundle_hash: Hash::zeros(),
            trunk_transaction: Hash::zeros(),
            branch_transaction: Hash::zeros(),
            nonce: String::default(),
            confirmed: true,
            broadcasted: true,
        }
    }

    #[tokio::test]
    async fn get_balance() {
        let account = _create_account();
        let address = _create_address();

        let response = super::get_balance(&account, &address).await;
        assert!(response.is_ok());
    }

    #[test]
    fn is_unspent_false() {
        let account = _create_account();
        let address = _create_address();

        let response = super::is_unspent(&account, &address);
        assert_eq!(response, false);
    }

    #[tokio::test]
    async fn is_unspent_true() {
        let mut account = _create_account();
        let address = super::get_new_address(&account).await.unwrap();
        let spent_tx = _generate_transaction(-50, address.clone());
        account.append_transactions(vec![spent_tx]);

        let response = super::is_unspent(&account, address.address());
        assert_eq!(response, true);
    }
}
