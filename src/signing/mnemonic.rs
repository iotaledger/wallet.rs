// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::account::Account;

use crypto::{
    hashes::{blake2b::Blake2b256, Digest},
    keys::{
        bip39::{mnemonic_to_seed, wordlist},
        slip10::{Chain, Curve, Seed},
    },
};
use iota_client::bee_message::{
    address::{Address, Ed25519Address},
    signature::{Ed25519Signature, SignatureUnlock},
    unlock::{ReferenceUnlock, UnlockBlock},
};
use once_cell::sync::OnceCell;

use std::{collections::HashMap, path::Path};

#[derive(Default)]
pub struct MnemonicSigner;

static MNEMONIC_SEED: OnceCell<[u8; 64]> = OnceCell::new();

/// Sets the mnemonic
pub fn set_mnemonic(mnemonic: String) -> crate::Result<()> {
    // first we check if the mnemonic is valid to give meaningful errors
    wordlist::verify(&mnemonic, &wordlist::ENGLISH).map_err(|e| crate::Error::InvalidMnemonic(format!("{:?}", e)))?;

    let mut mnemonic_seed = [0u8; 64];
    mnemonic_to_seed(&mnemonic, "", &mut mnemonic_seed);
    MNEMONIC_SEED
        .set(mnemonic_seed)
        .map_err(|_| crate::Error::MnemonicNotSet)?;
    Ok(())
}

/// Gets the mnemonic
pub(crate) fn get_mnemonic_seed() -> crate::Result<Seed> {
    Ok(Seed::from_bytes(
        MNEMONIC_SEED.get().ok_or(crate::Error::MnemonicNotSet)?,
    ))
}

fn generate_address(seed: &Seed, account_index: u32, address_index: u32, internal: bool) -> crate::Result<Address> {
    // 44 is for BIP 44 (HD wallets) and 4218 is the registered index for IOTA https://github.com/satoshilabs/slips/blob/master/slip-0044.md
    let chain = Chain::from_u32_hardened(vec![44, 4218, account_index, internal as u32, address_index]);
    let public_key = seed
        .derive(Curve::Ed25519, &chain)?
        .secret_key()
        .public_key()
        .to_bytes();
    // Hash the public key to get the address
    let result = Blake2b256::digest(&public_key)
        .try_into()
        .map_err(|_e| crate::Error::Blake2b256("Hashing the public key while generating the address failed."));

    Ok(Address::Ed25519(Ed25519Address::new(result?)))
}

#[async_trait::async_trait]
impl crate::signing::Signer for MnemonicSigner {
    async fn get_ledger_status(&self, _is_simulator: bool) -> crate::signing::LedgerStatus {
        // dummy status, function is only required in the trait because we need it for the LedgerSigner
        crate::signing::LedgerStatus {
            connected: false,
            locked: false,
            app: None,
        }
    }

    async fn store_mnemonic(&mut self, storage_path: &Path, mnemonic: String) -> crate::Result<()> {
        set_mnemonic(mnemonic)?;
        Ok(())
    }

    async fn generate_address(
        &mut self,
        account: &Account,
        address_index: usize,
        internal: bool,
        _: super::GenerateAddressMetadata,
    ) -> crate::Result<iota_client::bee_message::address::Address> {
        let seed = get_mnemonic_seed()?;
        generate_address(
            &seed,
            (*account.index()).try_into()?,
            address_index.try_into()?,
            internal,
        )
    }

    async fn sign_transaction<'a>(
        &mut self,
        account: &Account,
        essence: &iota_client::bee_message::prelude::Essence,
        inputs: &mut Vec<super::TransactionInput>,
        _: super::SignMessageMetadata<'a>,
    ) -> crate::Result<Vec<iota_client::bee_message::unlock::UnlockBlock>> {
        // order inputs https://github.com/luca-moser/protocol-rfcs/blob/signed-tx-payload/text/0000-transaction-payload/0000-transaction-payload.md
        inputs.sort_by(|a, b| a.input.cmp(&b.input));

        let hashed_essence = essence.hash();
        let mut unlock_blocks = Vec::new();
        let mut signature_indexes = HashMap::<String, usize>::new();

        for (current_block_index, input) in inputs.iter().enumerate() {
            // 44 is for BIP 44 (HD wallets) and 4218 is the registered index for IOTA https://github.com/satoshilabs/slips/blob/master/slip-0044.md
            let chain = Chain::from_u32_hardened(vec![
                44,
                4218,
                *account.index() as u32,
                input.address_internal as u32,
                input.address_index as u32,
            ]);
            // Check if current path is same as previous path
            // If so, add a reference unlock block
            // Format to differentiate between public and internal addresses
            let index = format!("{}{}", input.address_index, input.address_internal);
            if let Some(block_index) = signature_indexes.get(&index) {
                unlock_blocks.push(UnlockBlock::Reference(ReferenceUnlock::new(*block_index as u16)?));
            } else {
                // If not, we need to create a signature unlock block
                let private_key = get_mnemonic_seed()?.derive(Curve::Ed25519, &chain)?.secret_key();
                let public_key = private_key.public_key().to_bytes();
                // The signature unlock block needs to sign the hash of the entire transaction essence of the
                // transaction payload
                let signature = Box::new(private_key.sign(&hashed_essence).to_bytes());
                unlock_blocks.push(UnlockBlock::Signature(SignatureUnlock::Ed25519(Ed25519Signature::new(
                    public_key, *signature,
                ))));
                signature_indexes.insert(index, current_block_index);
            }
        }
        Ok(unlock_blocks)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn set_get_mnemonic() {
        let mnemonic = "giant dynamic museum toddler six deny defense ostrich bomb access mercy blood explain muscle shoot shallow glad autumn author calm heavy hawk abuse rally".to_string();
        let mut mnemonic_seed = [0u8; 64];
        crypto::keys::bip39::mnemonic_to_seed(&mnemonic, "", &mut mnemonic_seed);
        let _ = super::set_mnemonic(mnemonic.clone());
        let get_mnemonic_seed = super::get_mnemonic_seed().unwrap();
        // we can't compare `Seed`, that's why we generate an address and compare if it's the same
        assert_eq!(
            super::generate_address(&crypto::keys::slip10::Seed::from_bytes(&mnemonic_seed), 0, 0, false).unwrap(),
            super::generate_address(&get_mnemonic_seed, 0, 0, false).unwrap()
        );
    }

    #[tokio::test]
    async fn addresses() {
        #[cfg(feature = "events")]
        use crate::events::EventEmitter;
        use crate::{
            account::builder::AccountBuilder,
            signing::{GenerateAddressMetadata, Network, Signer, SignerType},
        };
        #[cfg(feature = "events")]
        use tokio::sync::Mutex;

        use std::path::Path;
        #[cfg(feature = "events")]
        use std::sync::Arc;

        let mnemonic = "giant dynamic museum toddler six deny defense ostrich bomb access mercy blood explain muscle shoot shallow glad autumn author calm heavy hawk abuse rally".to_string();
        let _ = super::MnemonicSigner.store_mnemonic(&Path::new(""), mnemonic).await;
        #[cfg(not(feature = "events"))]
        let account_handle = AccountBuilder::new(Default::default(), SignerType::Mnemonic)
            .finish()
            .await
            .unwrap();
        #[cfg(feature = "events")]
        let account_handle = AccountBuilder::new(
            Default::default(),
            SignerType::Mnemonic,
            Arc::new(Mutex::new(EventEmitter::new())),
        )
        .finish()
        .await
        .unwrap();
        let account = account_handle.read().await;
        let address = super::MnemonicSigner
            .generate_address(
                &account,
                0,
                false,
                GenerateAddressMetadata {
                    syncing: false,
                    network: Network::Testnet,
                },
            )
            .await
            .unwrap();

        assert_eq!(
            address.to_bech32("atoi"),
            "atoi1qpszqzadsym6wpppd6z037dvlejmjuke7s24hm95s9fg9vpua7vluehe53e".to_string()
        );
    }
}
