// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub(crate) mod account_recovery;
pub(crate) mod background_syncing;
pub(crate) mod get_account;
pub(crate) use account_recovery::recover_accounts;
pub(crate) use background_syncing::start_background_syncing;
pub(crate) use get_account::get_account;
