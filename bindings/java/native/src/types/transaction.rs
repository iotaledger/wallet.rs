// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_wallet::{
    message::{
        MessageTransactionPayload as MessageTransactionPayloadRust, TransactionEssence as TransactionEssenceRust,
        TransactionInput as RustWalletInput,
        TransactionRegularEssence as TransactionRegularEssenceRust,
    },
};

use crate::{
    Result,
    types::{MessagePayload, UnlockBlock, TransactionOutput}
};

use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq)]
pub enum InputKind {
    Utxo = 0,
    Treasury = 1,
}

pub struct TransactionPayload {
    essence: Essence,
    unlock_blocks: Vec<UnlockBlock>,
}

impl From<&Box<MessageTransactionPayloadRust>> for TransactionPayload {
    fn from(payload: &Box<MessageTransactionPayloadRust>) -> Self {
        Self {
            essence: Essence {
                essence: payload.essence().to_owned(),
            },
            unlock_blocks: payload
                .unlock_blocks()
                .iter()
                .cloned()
                .map(|unlock_block| unlock_block.into())
                .collect(),
        }
    }
}

impl TransactionPayload {
    pub fn essence(&self) -> Essence {
        self.essence.clone()
    }

    pub fn unlock_blocks(&self) -> Vec<UnlockBlock> {
        self.unlock_blocks.iter().cloned().collect()
    }
}

impl core::fmt::Display for TransactionPayload {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "essence={:?}, unlock_blocks=({:?})", self.essence, self.unlock_blocks)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Essence {
    essence: TransactionEssenceRust,
}

impl Essence {
    #[allow(irrefutable_let_patterns)]
    pub fn as_regular(&self) -> Result<RegularEssence> {
        if let TransactionEssenceRust::Regular(essence) = &self.essence {
            return Ok(RegularEssence {
                essence: essence.clone(),
            });
        };
        Err(anyhow::anyhow!("Essence is not of type Regular"))
    }
}

impl core::fmt::Display for Essence {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{:?}", self.essence)
    }
}

#[derive(Clone)]
pub struct RegularEssence {
    essence: TransactionRegularEssenceRust,
}

impl RegularEssence {
    pub fn inputs(&self) -> Vec<TransactionInput> {
        self.essence
            .inputs()
            .iter()
            .map(|input| input.into() )
            .collect()
    }

    /// Gets the transaction outputs.
    pub fn outputs(&self) -> Vec<TransactionOutput> {
        self.essence
            .outputs()
            .iter()
            .map(|output| output.into() )
            .collect()
    }

    /// Gets the transaction chained payload.
    pub fn payload(&self) -> Option<MessagePayload> {
        match self.essence.payload() {
            //Some(payload) => Some(payload.clone().into()),
            _ => None,
        }
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

impl core::fmt::Display for RegularEssence {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{:?}", self.essence)
    }
}

#[derive(Clone)]
pub struct TransactionInput {
    input: RustWalletInput,
}

impl TransactionInput {
    pub fn kind(&self) -> InputKind {
        match self.input {
            RustWalletInput::Utxo(_) => InputKind::Utxo,
            RustWalletInput::Treasury(_) => InputKind::Treasury,
        }
    }
}

impl Display for TransactionInput {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "({:?})", self.input)
    }
}

impl From<&RustWalletInput> for TransactionInput {
    fn from(input: &RustWalletInput) -> Self {
        Self { input: input.clone() }
    }
}