// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_wallet::{
    address::OutputKind as RustOutputKind,
    iota_client::block::prelude::{Payload as RustPayload, Unlock as RustUnlock},
    message::{
        InputSigningData as RustWalletInput, MessageTransactionPayload as MessageTransactionPayloadRust,
        TransactionEssence as TransactionEssenceRust, TransactionOutput as RustWalletOutput,
        TransactionRegularEssence as TransactionRegularEssenceRust,
    },
};

pub enum InputKind {
    Utxo = 0,
    Treasury = 1,
}

pub enum UnlockKind {
    Reference = 0,
    Ed25519 = 1,
}

pub struct MessageTransactionPayload {
    essence: Essence,
    unlocks: Vec<Unlock>,
}

impl From<&Box<MessageTransactionPayloadRust>> for MessageTransactionPayload {
    fn from(payload: &Box<MessageTransactionPayloadRust>) -> Self {
        Self {
            essence: Essence {
                essence: payload.essence().to_owned(),
            },
            unlocks: payload
                .unlocks()
                .iter()
                .cloned()
                .map(|unlock| Unlock { unlock: unlock })
                .collect(),
        }
    }
}

impl MessageTransactionPayload {
    pub fn essence(&self) -> Essence {
        self.essence.clone()
    }

    pub fn unlocks(&self) -> Vec<Unlock> {
        self.unlocks.iter().cloned().collect()
    }
}
#[derive(Clone)]
pub struct Essence {
    essence: TransactionEssenceRust,
}

impl Essence {
    #[allow(irrefutable_let_patterns)]
    pub fn get_as_regular(&self) -> Option<RegularEssence> {
        if let TransactionEssenceRust::Regular(essence) = &self.essence {
            return Some(RegularEssence {
                essence: essence.clone(),
            });
        };
        None
    }
}

#[derive(Clone)]
pub struct RegularEssence {
    essence: TransactionRegularEssenceRust,
}

impl RegularEssence {
    pub fn inputs(&self) -> Vec<InputSigningData> {
        self.essence
            .inputs()
            .iter()
            .cloned()
            .map(|input| InputSigningData { input: input })
            .collect()
    }

    /// Gets the transaction outputs.
    pub fn outputs(&self) -> Vec<TransactionOutput> {
        self.essence
            .outputs()
            .iter()
            .cloned()
            .map(|output| TransactionOutput { output: output })
            .collect()
    }

    /// Gets the transaction chained payload.
    pub fn payload(&self) -> &Option<RustPayload> {
        self.essence.payload()
    }

    /// Whether the transaction is between the mnemonic accounts or not.
    pub fn internal(&self) -> bool {
        self.essence.internal()
    }

    /// Whether the transaction is incoming or outgoing.
    pub fn incoming(&self) -> bool {
        self.essence.incoming()
    }

    /// The transactions's value.
    pub fn value(&self) -> u64 {
        self.essence.value()
    }

    /// The transactions's remainder value sum.
    pub fn remainder_value(&self) -> u64 {
        self.essence.remainder_value()
    }
}

#[derive(Clone)]
pub struct InputSigningData {
    input: RustWalletInput,
}

impl InputSigningData {
    pub fn kind(&self) -> InputKind {
        match self.input {
            RustWalletInput::Utxo(_) => InputKind::Utxo,
            RustWalletInput::Treasury(_) => InputKind::Treasury,
        }
    }

    pub fn to_string(&self) -> String {
        format!("{:?}", self.input)
    }
}

#[derive(Clone)]
pub struct TransactionOutput {
    output: RustWalletOutput,
}

impl TransactionOutput {
    pub fn kind(&self) -> RustOutputKind {
        match self.output {
            RustWalletOutput::SignatureLockedSingle(_) => RustOutputKind::SignatureLockedSingle,
            RustWalletOutput::SignatureLockedDustAllowance(_) => RustOutputKind::SignatureLockedDustAllowance,
            RustWalletOutput::Treasury(_) => RustOutputKind::Treasury,
        }
    }

    pub fn to_string(&self) -> String {
        format!("{:?}", self.output)
    }
}

#[derive(Clone)]
pub struct Unlock {
    unlock: RustUnlock,
}

impl Unlock {
    pub fn kind(&self) -> UnlockKind {
        match self.unlock {
            RustUnlock::Signature(_) => UnlockKind::Ed25519,
            RustUnlock::Reference(_) => UnlockKind::Reference,
        }
    }

    pub fn to_string(&self) -> String {
        format!("{:?}", self.unlock)
    }
}
