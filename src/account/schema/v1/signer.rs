// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", deny_unknown_fields)]
pub enum SignerType {
    /// Stronghold signer.
    Stronghold,
    /// Ledger Device
    LedgerNano,
    /// Ledger Speculos Simulator
    LedgerNanoSimulator,
    /// Custom signer with its identifier.
    Custom(String),
}
