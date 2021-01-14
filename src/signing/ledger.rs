// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::account::{Account};

use iota::{common::packable::Packable, UnlockBlock};
use std::{path::PathBuf};


pub const ED25519_ADDRESS_LENGTH: usize = 32;


const HARDENED : u32 = 0x80000000;

#[derive(Default)]
pub struct LedgerHardwareWalletSigner;

use ledger_iota::api::errors;

// map most errors to a single error but there are some errors that
// need special care.
// LedgerDongleLocked: Ask the user to unlock the dongle
// LedgerDeniedByUser: The user denied a signing
// LedgerDeviceNotFound: No usable Ledger device was found
// LedgerMiscError: Everything else.
fn ledger_map_err(err: errors::APIError) -> crate::Error {
    println!("error: {}", err);
    match err {
        errors::APIError::SecurityStatusNotSatisfied => {
            crate::Error::LedgerDongleLocked
        }
        errors::APIError::ConditionsOfUseNotSatisfied => {
            crate::Error::LedgerDeniedByUser
        }
        errors::APIError::TransportError => {
            crate::Error::LedgerDeviceNotFound
        }
        _ => {
            crate::Error::LedgerMiscError 
        }
    }
}

#[async_trait::async_trait]
impl super::Signer for LedgerHardwareWalletSigner {
    async fn store_mnemonic(&self, _: &PathBuf, mnemonic: String) -> crate::Result<()> {
        Ok(())        
    }

    async fn generate_address(
        &self,
        account: &Account,
        address_index: usize,
        internal: bool,
        meta: super::GenerateAddressMetadata, 
    ) -> crate::Result<iota::Address> {
        // get ledger
        let ledger = ledger_iota::get_ledger(false, *account.index() as u32).map_err(|e| ledger_map_err(e))?;

        let address_bytes = ledger.get_new_address(!meta.syncing, address_index as u32 | HARDENED)
            .map_err(|e| ledger_map_err(e))?;
        Ok(iota::Address::Ed25519(iota::Ed25519Address::new(address_bytes)))
    }

    async fn sign_message<'a>(
        &self,
        account: &Account,
        essence: &iota::TransactionEssence,
        inputs: &mut Vec<super::TransactionInput>,
        meta: super::SignMessageMetadata<'a>,
    ) -> crate::Result<Vec<iota::UnlockBlock>> {
        // get ledger
        let ledger = ledger_iota::get_ledger(false, *account.index() as u32).map_err(|e| ledger_map_err(e))?;

        // gather input indices into vec
        let mut key_indices : Vec<u32> = Vec::new();
        let input_len = inputs.len();
        for input in inputs {
            key_indices.push(input.address_index as u32 | HARDENED);
        }

        // figure out the remainder address and bip32 index
        let (remainder_address, remainder_bip32) : (&iota::Address, u32) = match meta.remainder_deposit_address {
            Some(a) => {
                (a.address(), *a.key_index() as u32 | HARDENED)
            }
            None => {
                return Err(crate::Error::LedgerMiscError);
            }
        };
        let remainder_bytes = remainder_address.pack_new();

        // find the index in the essence
        // it is verified on the ledger nano s/x in any case
        let mut remainder_index = 0;
        for output in essence.outputs().iter() {
            match output {
                iota::Output::SignatureLockedSingle(s) => {
                    if *remainder_address == *s.address() {
                        break;
                    }
                }
                _ => {
                    return Err(crate::Error::LedgerMiscError);
                }
            }
            remainder_index += 1; 
        }

        // was index found?
        if remainder_index as usize == essence.outputs().len() {
            return Err(crate::Error::LedgerMiscError);
        }
    
        // pack essence into bytes
        let essence_bytes = essence.pack_new();

        // prepare signing 
        ledger.prepare_signing(
                key_indices, 
                essence_bytes, 
                remainder_index, 
                remainder_bip32)
            .map_err(|e| ledger_map_err(e))?;

        // show essence to user
        // if denied by user, it returns with `DeniedByUser` Error
        ledger.user_confirm()
            .map_err(|e| ledger_map_err(e))?;


        // sign
        let signature_bytes = ledger.sign(input_len as u16)
            .map_err(|e| ledger_map_err(e))?;
        

        // unpack signature to unlockblocks
        let mut unlock_blocks = Vec::new();
        for _ in 0..input_len { 
            unlock_blocks.push(UnlockBlock::unpack(&mut &signature_bytes[..])?);
        }
        Ok(unlock_blocks)
    }
}
