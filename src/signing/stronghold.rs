use crate::account::{account_id_to_stronghold_record_id, Account};

use std::convert::TryInto;

#[derive(Default)]
pub struct StrongholdSigner;

impl super::Signer for StrongholdSigner {
  fn init_account(&self, account: &Account, mnemonic: Option<String>) -> crate::Result<String> {
    let stronghold_account_res: crate::Result<stronghold::Account> =
      crate::with_stronghold_from_path(account.storage_path(), |stronghold| {
        let created_at_timestamp: u128 = account.created_at().timestamp().try_into().unwrap(); // safe to unwrap since it's > 0;
        let account = match mnemonic {
          Some(mnemonic) => stronghold.account_import(
            Some(created_at_timestamp),
            Some(created_at_timestamp),
            mnemonic,
            Some("password"),
          )?,
          None => stronghold.account_create(Some("password".to_string()))?,
        };
        Ok(account)
      });
    let stronghold_account = stronghold_account_res?;
    let id = stronghold_account.id();
    Ok(hex::encode(id))
  }

  fn generate_address(
    &self,
    account: &Account,
    address_index: usize,
    internal: bool,
  ) -> crate::Result<iota::Address> {
    crate::with_stronghold_from_path(account.storage_path(), |stronghold| {
      let address_str = stronghold.address_get(
        &account_id_to_stronghold_record_id(account.id())?,
        Some(*account.index()),
        address_index,
        internal,
      )?;
      crate::address::parse(address_str)
    })
  }

  fn sign_message(
    &self,
    account: &Account,
    essence: &iota::TransactionEssence,
    inputs: &mut Vec<super::TransactionInput>,
  ) -> crate::Result<Vec<iota::UnlockBlock>> {
    let mut inputs = inputs
      .iter_mut()
      .map(|i| stronghold::AddressIndexRecorder {
        input: i.input.clone(),
        address_index: i.address_index,
        address_path: i.address_path.clone(),
      })
      .collect::<Vec<stronghold::AddressIndexRecorder>>();
    crate::with_stronghold_from_path(account.storage_path(), |stronghold| {
      stronghold.get_transaction_unlock_blocks(
        &account_id_to_stronghold_record_id(account.id())?,
        &essence,
        &mut inputs,
      )
    })
    .map_err(|e| e.into())
  }
}
