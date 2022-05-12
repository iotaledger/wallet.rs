// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_wallet::{
    iota_client::bee_message::{
        payload::{milestone::MilestoneId, transaction::TransactionId},
        prelude::{TreasuryInput as RustTreasuryInput, UtxoInput as RustUtxoInput},
    },
    message::{TransactionInput as RustInput, TransactionUtxoInput as RustTransactionUtxoInput},
};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

use crate::Result;

#[derive(Debug, Clone, PartialEq)]
pub enum InputKind {
    Utxo = 0,
    Treasury = 1,
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct TransactionInput(RustInput);

impl TransactionInput {
    pub fn kind(&self) -> InputKind {
        match self.0 {
            RustInput::Utxo(_) => InputKind::Utxo,
            RustInput::Treasury(_) => InputKind::Treasury,
        }
    }

    pub fn as_utxo(&self) -> Result<UtxoInput> {
        if let RustInput::Utxo(payload) = &self.0 {
            Ok(payload.clone().into())
        } else {
            Err(anyhow::anyhow!("Input is not of type Utxo"))
        }
    }

    pub fn as_treasury(&self) -> Result<TreasuryInput> {
        if let RustInput::Treasury(payload) = self.0 {
            Ok(payload.clone().into())
        } else {
            Err(anyhow::anyhow!("Input is not of type Treasury"))
        }
    }

    pub fn to_inner_clone(&self) -> RustInput {
        self.0.clone()
    }
}

impl From<&RustInput> for TransactionInput {
    fn from(input: &RustInput) -> Self {
        Self(input.clone())
    }
}

impl Display for TransactionInput {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "({:?})", self.0)
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct UtxoInput(RustTransactionUtxoInput);

impl UtxoInput {
    pub fn from(id: TransactionId, index: u16) -> Result<Self> {
        match RustUtxoInput::new(id, index) {
            Ok(e) => Ok(Self(RustTransactionUtxoInput {
                input: e,
                metadata: None,
            })),
            Err(e) => Err(anyhow::anyhow!(e.to_string())),
        }
    }

    /// Returns the `TransactionId` of an `OutputId`.
    pub fn transaction_id(&self) -> TransactionId {
        self.0.input.output_id().transaction_id().clone()
    }

    /// Returns the index of an `OutputId`.
    pub fn index(&self) -> u16 {
        self.0.input.output_id().index()
    }

    pub fn to_inner_clone(&self) -> RustTransactionUtxoInput {
        self.0.clone()
    }
}
impl Display for UtxoInput {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "(transaction_id={}, index={})", self.transaction_id(), self.index())
    }
}

impl From<RustTransactionUtxoInput> for UtxoInput {
    fn from(input: RustTransactionUtxoInput) -> Self {
        Self(input)
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct TreasuryInput(RustTreasuryInput);

impl TreasuryInput {
    pub fn new(id: MilestoneId) -> Self {
        Self(RustTreasuryInput::new(id))
    }

    pub fn milestone_id(&self) -> MilestoneId {
        *self.0.milestone_id()
    }

    pub fn to_inner_clone(&self) -> RustTreasuryInput {
        self.0
    }
}

impl From<RustTreasuryInput> for TreasuryInput {
    fn from(input: RustTreasuryInput) -> Self {
        Self(input)
    }
}

impl Display for TreasuryInput {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "({:?})", self.0)
    }
}
