// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::account::Account;

use iota::{common::packable::Packable, UnlockBlock};
use std::path::PathBuf;

use ledger_iota::LedgerHardwareWallet;

use ledger_iota::{api::errors, LedgerBIP32Index};

const HARDENED: u32 = 0x80000000;

pub struct LedgerNanoSigner {
    pub id: u64,
    pub account: u32,
    pub ledger: Option<Box<LedgerHardwareWallet>>,
    pub is_simulator: bool,
}

/// A record matching an Input with its address.
#[derive(Debug)]
struct AddressIndexRecorder {
    /// the input
    pub input: iota::Input,

    /// bip32 index
    pub bip32: LedgerBIP32Index,
}

// map most errors to a single error but there are some errors that
// need special care.
// LedgerDongleLocked: Ask the user to unlock the dongle
// LedgerDeniedByUser: The user denied a signing
// LedgerDeviceNotFound: No usable Ledger device was found
// LedgerMiscError: Everything else.
// LedgerEssenceTooLarge: Essence with bip32 input indices need more space then the internal buffer is big
fn ledger_map_err(err: errors::APIError) -> crate::Error {
    log::info!("ledger error: {}", err);
    match err {
        errors::APIError::SecurityStatusNotSatisfied => crate::Error::LedgerDongleLocked,
        errors::APIError::ConditionsOfUseNotSatisfied => crate::Error::LedgerDeniedByUser,
        errors::APIError::TransportError => crate::Error::LedgerDeviceNotFound,
        errors::APIError::EssenceTooLarge => crate::Error::LedgerEssenceTooLarge,
        _ => crate::Error::LedgerMiscError,
    }
}

impl LedgerNanoSigner {
    pub fn new(id: u64, is_simulator: bool) -> Self {
        LedgerNanoSigner {
            account: 0u32,
            id,
            is_simulator,
            ledger: None,
        }
    }

    /// Init the ledger object or get it. The `account` is set only once at init, that means generating addresses for
    /// different accounts with a single signer object instance is intentionally not allowed.
    fn get_or_init(&mut self, account: u32) -> crate::Result<&mut LedgerHardwareWallet> {
        match &self.ledger {
            None => {
                self.ledger =
                    Some(ledger_iota::get_ledger(self.id, self.is_simulator, account).map_err(ledger_map_err)?);
                self.account = account;
            }
            _ => {
                if account != self.account {
                    log::error!("account index changed from {} to {}!", self.account, account);
                    return Err(crate::Error::LedgerMiscError);
                }
            }
        };
        Ok(self.ledger.as_mut().unwrap())
    }
}

#[async_trait::async_trait]
impl super::Signer for LedgerNanoSigner {
    async fn store_mnemonic(&mut self, _: &PathBuf, _mnemonic: String) -> crate::Result<()> {
        Err(crate::Error::InvalidMnemonic(String::from("")))
    }

    async fn generate_address(
        &mut self,
        account: &Account,
        address_index: usize,
        internal: bool,
        meta: super::GenerateAddressMetadata,
    ) -> crate::Result<iota::Address> {
        let ledger = self.get_or_init(*account.index() as u32 | HARDENED)?;

        let bip32 = ledger_iota::LedgerBIP32Index {
            bip32_index: address_index as u32 | HARDENED,
            bip32_change: if internal { 1 } else { 0 } | HARDENED,
        };

        // if the wallet is not generating addresses for syncing, we assume it's a new receiving address that
        // needs to be shown to the user
        let address_bytes = ledger.get_new_address(!meta.syncing, bip32).map_err(ledger_map_err)?;

        Ok(iota::Address::Ed25519(iota::Ed25519Address::new(address_bytes)))
    }

    async fn sign_message<'a>(
        &mut self,
        account: &Account,
        essence: &iota::TransactionPayloadEssence,
        inputs: &mut Vec<super::TransactionInput>,
        meta: super::SignMessageMetadata<'a>,
    ) -> crate::Result<Vec<iota::UnlockBlock>> {
        let ledger = self.get_or_init(*account.index() as u32 | HARDENED)?;

        let input_len = inputs.len();

        // on essence finalization, inputs are sorted lexically before they are packed into bytes.
        // we need the correct order of the bip32 indices before we can call PrepareSigning, but
        // because inputs of the essence don't have bip32 indices, we need to sort it on our own too.
        let mut address_index_recorders: Vec<AddressIndexRecorder> = Vec::new();
        for input in inputs {
            address_index_recorders.push(AddressIndexRecorder {
                input: input.input.clone(),
                bip32: LedgerBIP32Index {
                    bip32_index: input.address_index as u32 | HARDENED,
                    bip32_change: if input.address_internal { 1 } else { 0 } | HARDENED,
                },
            });
        }
        address_index_recorders.sort_by(|a, b| a.input.cmp(&b.input));

        // now extract the bip32 indices in the right order
        let mut input_bip32_indices: Vec<LedgerBIP32Index> = Vec::new();
        for recorder in address_index_recorders {
            input_bip32_indices.push(recorder.bip32);
        }

        // figure out the remainder address and bip32 index (if there is one)
        let (has_remainder, remainder_address, remainder_bip32): (bool, Option<&iota::Address>, LedgerBIP32Index) =
            match meta.remainder_deposit_address {
                Some(a) => (
                    true,
                    Some(a.address().as_ref()),
                    LedgerBIP32Index {
                        bip32_index: *a.key_index() as u32 | HARDENED,
                        bip32_change: if *a.internal() { 1 } else { 0 } | HARDENED,
                    },
                ),
                None => (false, None, LedgerBIP32Index::default()),
            };

        let mut remainder_index = 0u16;
        if has_remainder {
            // find the index of the remainder in the essence
            // this has to be done because outputs in essences are sorted
            // lexically and therefore the remainder is not always the last output.
            // The index within the essence and the bip32 index will be validated
            // by the hardware wallet.
            // The outputs in the essence already are sorted (done by `essence_builder.finish`)
            // at this place, so we can rely on their order and don't have to sort it again.
            for output in essence.outputs().iter() {
                match output {
                    iota::Output::SignatureLockedSingle(s) => {
                        if *remainder_address.unwrap() == *s.address() {
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
        }

        // pack essence into bytes
        let essence_bytes = essence.pack_new();

        // prepare signing
        ledger
            .prepare_signing(
                input_bip32_indices,
                essence_bytes,
                has_remainder,
                remainder_index,
                remainder_bip32,
            )
            .map_err(ledger_map_err)?;

        // show essence to user
        // if denied by user, it returns with `DeniedByUser` Error
        ledger.user_confirm().map_err(ledger_map_err)?;

        // sign
        let signature_bytes = ledger.sign(input_len as u16).map_err(ledger_map_err)?;
        let mut readable = &mut &*signature_bytes;
        // unpack signature to unlockblocks
        let mut unlock_blocks = Vec::new();
        for _ in 0..input_len {
            let unlock_block = UnlockBlock::unpack(&mut readable)?;
            unlock_blocks.push(unlock_block);
        }
        Ok(unlock_blocks)
    }
}
