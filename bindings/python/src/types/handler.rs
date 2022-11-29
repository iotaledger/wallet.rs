// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_wallet::message_interface::WalletMessageHandler as RustWalletMessageHandler;
use pyo3::prelude::*;

#[pyclass]
pub struct WalletMessageHandler {
    pub wallet_message_handler: RustWalletMessageHandler,
}
