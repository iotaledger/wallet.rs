// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_wallet::{account_manager::AccountManager, iota_client::block::protocol::ProtocolParameters};

pub struct Context {
    pub account_manager: AccountManager,
    pub protocol_parameters: ProtocolParameters,
}
