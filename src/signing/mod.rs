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

use std::{
  collections::HashMap,
  sync::{Arc, RwLock},
};

use crate::account::Account;
use iota::Input;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use slip10::BIP32Path;

mod stronghold;
use self::stronghold::StrongholdSigner;

type BoxedSigner = Box<dyn Signer + Sync + Send>;
type Signers = Arc<RwLock<HashMap<SignerType, BoxedSigner>>>;
static SIGNERS_INSTANCE: OnceCell<Signers> = OnceCell::new();

/// The signer types.
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum SignerType {
  /// Stronghold signer.
  #[cfg(feature = "stronghold")]
  Stronghold,
  /// Custom signer with its identifier.
  Custom(String),
}

/// One of the transaction inputs and its address information needed for signing it.
pub struct TransactionInput {
  /// The input.
  pub input: Input,
  /// Input's address index.
  pub address_index: usize,
  /// Input's address BIP32 derivation path.
  pub address_path: BIP32Path,
}

/// Signer interface.
pub trait Signer {
    /// Initialises an account.
    fn init_account(&self, account: &Account, mnemonic: Option<String>) -> crate::Result<String>;
    /// Generates an address.
    fn generate_address(
        &self,
        account: &Account,
        index: usize,
        internal: bool,
    ) -> crate::Result<iota::Address>;
    /// Signs message.
    fn sign_message(
        &self,
        account: &Account,
        essence: &iota::TransactionEssence,
        inputs: &mut Vec<TransactionInput>,
    ) -> crate::Result<Vec<iota::UnlockBlock>>;
}

#[allow(unused_mut)]
fn default_signers() -> Signers {
  let mut signers = HashMap::new();

  #[cfg(feature = "stronghold")]
  {
    signers.insert(
      SignerType::Stronghold,
      Box::new(StrongholdSigner::default()) as Box<dyn Signer + Sync + Send>,
    );
  }

  Arc::new(RwLock::new(signers))
}

/// Sets the signer interface for the given type.
pub fn set_signer<S: Signer + Sync + Send + 'static>(signer_type: SignerType, signer: S) {
  let mut instances = SIGNERS_INSTANCE
    .get_or_init(default_signers)
    .write()
    .unwrap();
  instances.insert(signer_type, Box::new(signer));
}

/// Gets the signer interface.
pub(crate) fn with_signer<T, F: FnOnce(&BoxedSigner) -> T>(signer_type: &SignerType, cb: F) -> T {
  let instances = SIGNERS_INSTANCE
    .get_or_init(default_signers)
    .read()
    .unwrap();
  if let Some(instance) = instances.get(signer_type) {
    cb(instance)
  } else {
    panic!(format!("signer not initialized for type {:?}", signer_type))
  }
}
