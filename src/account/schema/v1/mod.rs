// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub(super) mod address;
pub(super) mod client;
pub(super) mod signer;

use address::Address;
use client::ClientOptions;
use signer::SignerType;

use serde::{Deserialize, Serialize};

/// Account definition.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Account {
    /// The account identifier.
    pub(super) id: String,
    /// The account's signer type.
    pub(super) signer_type: SignerType,
    /// The account index
    pub(super) index: usize,
    /// The account alias.
    pub(super) alias: String,
    /// Time of account creation.
    pub(super) created_at: String,
    /// Time the account was last synced with the Tangle.
    pub(super) last_synced_at: Option<String>,
    /// Address history associated with the seed.
    pub(super) addresses: Vec<Address>,
    /// The client options.
    pub(super) client_options: ClientOptions,
    pub(super) storage_path: String,
}
