// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::account::Account;

use std::{collections::HashMap, env, fs::OpenOptions, io::Write, path::PathBuf};

use bech32::ToBase32;
use blake2::{
    digest::{Update, VariableOutput},
    VarBlake2b,
};
use dialoguer::Confirm;
use hmac::Hmac;
use iota::{common::packable::Packable, Ed25519Signature, ReferenceUnlock, SignatureUnlock, UnlockBlock};
use unicode_normalization::UnicodeNormalization;

use bee_signing_ext::{
    binary::{ed25519, BIP32Path},
    Signer,
};

const MNEMONIC_ENV_KEY: &str = "IOTA_WALLET_MNEMONIC";
const MNEMONIC_PASSWORD_ENV_KEY: &str = "IOTA_WALLET_MNEMONIC_PASSWORD";
const PBKDF2_ROUNDS: u32 = 2048;
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
        let _ = dotenv::dotenv();
        let mnemonic = env::var(MNEMONIC_ENV_KEY).expect("must set the IOTA_WALLET_MNEMONIC environment variable");
        bip39::Mnemonic::validate(mnemonic.as_str(), bip39::Language::English).expect("invalid BIP-39 mnemonic");
        mnemonic_to_ed25_seed(
            mnemonic,
            env::var(MNEMONIC_PASSWORD_ENV_KEY).unwrap_or_else(|_| "password".to_string()),
        )
    }

    fn get_private_key(&self, derivation_path: String) -> crate::Result<ed25519::Ed25519PrivateKey> {
        let seed = self.get_seed();
        let derivation_path = BIP32Path::from_str(&derivation_path)
            .map_err(|_| crate::Error::InvalidDerivationPath(derivation_path.clone()))?;
        Ok(ed25519::Ed25519PrivateKey::generate_from_seed(&seed, &derivation_path)
            .map_err(|_| crate::Error::FailedToGeneratePrivateKey(derivation_path))?)
    }
}

#[async_trait::async_trait]
impl super::Signer for EnvMnemonicSigner {
    async fn store_mnemonic(&self, _: &PathBuf, mnemonic: String) -> crate::Result<()> {
        // if the mnemonic is already on the env, we skip the logging and prompting processes
        if mnemonic != env::var(MNEMONIC_ENV_KEY).unwrap_or_default() {
            env::set_var(MNEMONIC_ENV_KEY, &mnemonic);
            println!("Your mnemonic is `{}`, you must store it on an environment variable called `IOTA_WALLET_MNEMONIC` to use this CLI", mnemonic);
            if let Ok(flag) = Confirm::new()
                .with_prompt("Do you want to store the mnemonic in a .env file?")
                .interact()
            {
                if flag {
                    let mut file = OpenOptions::new().append(true).create(true).open(".env")?;
                    writeln!(file, r#"IOTA_WALLET_MNEMONIC="{}""#, mnemonic)?;
                    println!("mnemonic added to {:?}", std::env::current_dir()?.join(".env"));
                }
            }
        }
        Ok(())
    }

    async fn generate_address(
        &self,
        account: &Account,
        address_index: usize,
        internal: bool,
        _: super::GenerateAddressMetadata,
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

    async fn sign_message<'a>(
        &self,
        _account: &Account,
        essence: &iota::TransactionPayloadEssence,
        inputs: &mut Vec<super::TransactionInput>,
        _: super::SignMessageMetadata<'a>,
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
                unlock_blocks.push(UnlockBlock::Reference(ReferenceUnlock::new(*block_index as u16)?));
            } else {
                // If not, we should create a signature unlock block
                let private_key = ed25519::Ed25519PrivateKey::generate_from_seed(&seed, &recorder.address_path)
                    .map_err(|_| crate::Error::FailedToGeneratePrivateKey(recorder.address_path.clone()))?;
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
