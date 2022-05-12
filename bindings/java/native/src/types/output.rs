// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_wallet::{
    address::{AddressWrapper, OutputKind as RustOutputKind},
    iota_client::bee_message::output::TreasuryOutput as RustTreasuryOutput,
    message::{
        TransactionOutput as RustOutput,
        TransactionSignatureLockedDustAllowanceOutput as RustSignatureLockedDustAllowanceOutput,
        TransactionSignatureLockedSingleOutput as RustSignatureLockedSingleOutput,
    },
};

use crate::Result;
use anyhow::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq)]
pub enum OutputKind {
    None = 0,
    SignatureLockedSingle = 1,
    SignatureLockedDustAllowance = 2,
    Treasury = 3,
}

impl From<RustOutputKind> for OutputKind {
    fn from(ouput: RustOutputKind) -> Self {
        match ouput {
            RustOutputKind::SignatureLockedSingle => OutputKind::SignatureLockedSingle,
            RustOutputKind::SignatureLockedDustAllowance => OutputKind::SignatureLockedDustAllowance,
            RustOutputKind::Treasury => OutputKind::Treasury,
        }
    }
}

pub fn output_kind_enum_to_type(strategy: OutputKind) -> Option<RustOutputKind> {
    match strategy {
        OutputKind::None => None,
        OutputKind::SignatureLockedSingle => Some(RustOutputKind::SignatureLockedSingle),
        OutputKind::SignatureLockedDustAllowance => Some(RustOutputKind::SignatureLockedDustAllowance),
        OutputKind::Treasury => Some(RustOutputKind::Treasury),
    }
}

pub struct TransactionOutput {
    output: RustOutput,
}

impl TransactionOutput {
    pub fn kind(&self) -> OutputKind {
        match self.output {
            RustOutput::SignatureLockedSingle(_) => OutputKind::SignatureLockedSingle,
            RustOutput::SignatureLockedDustAllowance(_) => OutputKind::SignatureLockedDustAllowance,
            RustOutput::Treasury(_) => OutputKind::Treasury,
        }
    }

    pub fn as_signature_locked_single_output(&self) -> Result<SignatureLockedSingleOutput> {
        SignatureLockedSingleOutput::try_from(self.output.clone())
    }

    pub fn as_signature_locked_dust_allowance_output(&self) -> Result<SignatureLockedDustAllowanceOutput> {
        SignatureLockedDustAllowanceOutput::try_from(self.output.clone())
    }

    pub fn as_treasury_output(&self) -> Result<TreasuryOutput> {
        TreasuryOutput::try_from(self.output.clone())
    }
}

impl Display for TransactionOutput {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "({:?})", self.output)
    }
}

impl From<&RustOutput> for TransactionOutput {
    fn from(output: &RustOutput) -> Self {
        Self { output: output.clone() }
    }
}

/// Describes a deposit to a single address which is unlocked via a signature.
#[derive(Clone, Debug)]
pub struct SignatureLockedSingleOutput(RustSignatureLockedSingleOutput);

impl SignatureLockedSingleOutput {
    pub(crate) fn from_rust(output: RustSignatureLockedSingleOutput) -> Self {
        Self(output)
    }

    pub fn from(address: AddressWrapper, amount: u64, remainder: bool) -> SignatureLockedSingleOutput {
        Self(RustSignatureLockedSingleOutput::from(address, amount, remainder))
    }
    pub fn amount(&self) -> u64 {
        self.0.amount()
    }
    pub fn address(&self) -> AddressWrapper {
        self.0.address().clone().into()
    }

    pub fn to_inner_clone(&self) -> RustSignatureLockedSingleOutput {
        self.0.clone()
    }
}

impl TryFrom<RustOutput> for SignatureLockedSingleOutput {
    type Error = Error;
    fn try_from(output: RustOutput) -> Result<Self, Self::Error> {
        match output {
            RustOutput::SignatureLockedSingle(ed) => Ok(Self(ed)),
            _ => unimplemented!(),
        }
    }
}

impl Display for SignatureLockedSingleOutput {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "({:?})", self.0)
    }
}

/// Output type for deposits that enables an address to receive dust outputs. It can be consumed as an input like a
/// regular SigLockedSingleOutput.
#[derive(Clone, Debug)]
pub struct SignatureLockedDustAllowanceOutput(RustSignatureLockedDustAllowanceOutput);
impl SignatureLockedDustAllowanceOutput {
    pub fn from(address: AddressWrapper, amount: u64) -> SignatureLockedDustAllowanceOutput {
        Self(RustSignatureLockedDustAllowanceOutput::from(address, amount))
    }

    pub fn amount(&self) -> u64 {
        *self.0.amount()
    }

    pub fn address(&self) -> AddressWrapper {
        self.0.address().clone().into()
    }
    pub fn to_inner_clone(&self) -> RustSignatureLockedDustAllowanceOutput {
        self.0.clone()
    }
}

impl TryFrom<RustOutput> for SignatureLockedDustAllowanceOutput {
    type Error = Error;
    fn try_from(output: RustOutput) -> Result<Self, Self::Error> {
        match output {
            RustOutput::SignatureLockedDustAllowance(ed) => Ok(Self(ed)),
            _ => unimplemented!(),
        }
    }
}

impl Display for SignatureLockedDustAllowanceOutput {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "({:?})", self.0)
    }
}

#[derive(Clone, Debug)]
pub struct TreasuryOutput(RustTreasuryOutput);

impl TreasuryOutput {
    pub fn from(amount: u64) -> Result<TreasuryOutput> {
        match RustTreasuryOutput::new(amount) {
            Ok(e) => Ok(Self(e)),
            Err(e) => Err(anyhow::anyhow!(e.to_string())),
        }
    }

    pub fn amount(&self) -> u64 {
        self.0.amount()
    }
    pub fn to_inner_clone(&self) -> RustTreasuryOutput {
        self.0.clone()
    }
}

impl TryFrom<RustOutput> for TreasuryOutput {
    type Error = Error;
    fn try_from(output: RustOutput) -> Result<Self, Self::Error> {
        match output {
            RustOutput::Treasury(ed) => Ok(Self(ed)),
            _ => unimplemented!(),
        }
    }
}
impl From<RustTreasuryOutput> for TreasuryOutput {
    fn from(output: RustTreasuryOutput) -> Self {
        Self(output)
    }
}
impl Display for TreasuryOutput {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "({:?})", self.0)
    }
}
