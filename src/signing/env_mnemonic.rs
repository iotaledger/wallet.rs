// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::account::Account;

use std::{collections::HashMap, env};

use bech32::ToBase32;
use bee_common::packable::Packable;
use blake2::{
    digest::{Update, VariableOutput},
    VarBlake2b,
};
use hmac::Hmac;
use iota::{Ed25519Signature, ReferenceUnlock, SignatureUnlock, UnlockBlock};
use rand::{thread_rng, Rng};
use unicode_normalization::UnicodeNormalization;

use bee_signing_ext::{
    binary::{ed25519, BIP32Path},
    Signer,
};

const PBKDF2_ROUNDS: usize = 2048;
const PBKDF2_BYTES: usize = 32; // 64 for secp256k1 , 32 for ed25

/// PBKDF2 helper, used to generate [`Seed`][Seed] from [`Mnemonic`][Mnemonic]
///
/// [Mnemonic]: ../mnemonic/struct.Mnemonic.html
/// [Seed]: ../seed/struct.Seed.html
fn _pbkdf2(input: &[u8], salt: &str) -> Vec<u8> {
    let mut seed = vec![0u8; PBKDF2_BYTES];
    pbkdf2::pbkdf2::<Hmac<sha2::Sha512>>(input, salt.as_bytes(), PBKDF2_ROUNDS, &mut seed);
    seed
}

fn mnemonic_to_ed25_seed(mnemonic: String, password: String) -> ed25519::Ed25519Seed {
    let salt = format!("mnemonic{}", password);
    let normalized_salt = salt.nfkd().to_string();
    let bytes = _pbkdf2(mnemonic.as_bytes(), &normalized_salt);
    ed25519::Ed25519Seed::from_bytes(&bytes).unwrap()
}

fn derive_into_address(private_key: ed25519::Ed25519PrivateKey) -> String {
    let public_key = private_key.generate_public_key().to_bytes();
    // Hash the public key to get the address
    let mut hasher = VarBlake2b::new(32).unwrap();
    hasher.update(public_key);
    let mut result = vec![1];
    hasher.finalize_variable(|res| {
        result.extend(res.to_vec());
    });

    bech32::encode("iot", result.to_base32()).unwrap()
}

#[derive(Default)]
pub struct EnvMnemonicSigner;

impl EnvMnemonicSigner {
    fn get_seed(&self) -> ed25519::Ed25519Seed {
        mnemonic_to_ed25_seed(
            env::var("IOTA_WALLET_MNEMONIC").expect("must set the IOTA_WALLET_MNEMONIC environment variable"),
            env::var("IOTA_WALLET_MNEMONIC_PASSWORD").unwrap_or_else(|_| "password".to_string()),
        )
    }

    fn get_private_key(&self, derivation_path: String) -> crate::Result<ed25519::Ed25519PrivateKey> {
        let seed = self.get_seed();
        Ok(ed25519::Ed25519PrivateKey::generate_from_seed(
            &seed,
            &BIP32Path::from_str(&derivation_path).map_err(|e| anyhow::anyhow!(e.to_string()))?,
        )
        .map_err(|e| anyhow::anyhow!(e.to_string()))?)
    }
}

impl super::Signer for EnvMnemonicSigner {
    fn init_account(&self, account: &Account, mnemonic: Option<String>) -> crate::Result<String> {
        if let Some(mnemonic) = mnemonic {
            env::set_var("IOTA_WALLET_MNEMONIC", mnemonic);
        }
        Ok(thread_rng().gen_ascii_chars().take(10).collect())
    }

    fn generate_address(
        &self,
        account: &Account,
        address_index: usize,
        internal: bool,
    ) -> crate::Result<iota::Address> {
        let private_key = self.get_private_key(format!(
            "m/44H/4218H/{}H/{}H/{}H",
            account.index(),
            internal as u32,
            address_index
        ))?;
        let address_str = derive_into_address(private_key);
        crate::address::parse(address_str)
    }

    fn sign_message(
        &self,
        account: &Account,
        essence: &iota::TransactionEssence,
        inputs: &mut Vec<super::TransactionInput>,
    ) -> crate::Result<Vec<iota::UnlockBlock>> {
        let serialized_essence = essence.pack_new();

        let seed = self.get_seed();
        let mut unlock_blocks = vec![];
        let mut current_block_index: usize = 0;
        let mut signature_indexes = HashMap::<usize, usize>::new();
        inputs.sort_by(|a, b| a.input.cmp(&b.input));

        for recorder in inputs.iter() {
            // Check if current path is same as previous path
            // If so, add a reference unlock block
            if let Some(block_index) = signature_indexes.get(&recorder.address_index) {
                unlock_blocks.push(UnlockBlock::Reference(
                    ReferenceUnlock::new(*block_index as u16)
                        .map_err(|e| anyhow::anyhow!("failed to create reference unlock block"))?,
                ));
            } else {
                // If not, we should create a signature unlock block
                let private_key = ed25519::Ed25519PrivateKey::generate_from_seed(&seed, &recorder.address_path)
                    .map_err(|_| anyhow::anyhow!("invalid parameter: seed inputs"))?;
                let public_key = private_key.generate_public_key().to_bytes();
                // The block should sign the entire transaction essence part of the transaction payload
                let signature = Box::new(private_key.sign(&serialized_essence).to_bytes());
                unlock_blocks.push(UnlockBlock::Signature(SignatureUnlock::Ed25519(Ed25519Signature::new(
                    public_key, signature,
                ))));
                signature_indexes.insert(recorder.address_index, current_block_index);

                // Update current block index
                current_block_index += 1;
            }
        }
        Ok(unlock_blocks)
    }
}
