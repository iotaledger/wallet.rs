// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! The IOTA Wallet Library

/// [`AccountHandle`]: crate::account::handle::AccountHandle
/// The account module. Interaction with an Account happens via an [`AccountHandle`].
pub mod account;
/// The account manager module.
pub mod account_manager;
/// The message passing interface for the library. A different way to call the wallet functions, useful for bindings to
/// other languages.
#[cfg(feature = "message_interface")]
pub mod message_interface;

/// The ClientOptions to build the iota_client for interactions with the IOTA Tangle.
pub use iota_client::ClientBuilder as ClientOptions;

/// The error module.
pub mod error;
#[cfg(feature = "events")]
/// The event module.
pub mod events;

// todo: add Stronghold support, refactor, check if it can't lock funds if it's stored with locked outputs and
// transaction creation failed so they should be unlocked again and other edge cases
#[cfg(feature = "storage")]
/// The storage module.
pub mod storage;

/// Module for debug logs.
pub mod logger;

// Expose for high level functions
pub use iota_client::{self, signing};
pub use primitive_types::U256;

pub use self::{
    account::operations::transfer::high_level::{
        mint_native_token::NativeTokenOptions, mint_nfts::NftOptions, send_amount::AddressWithAmount,
        send_micro_transaction::AddressWithMicroAmount, send_native_tokens::AddressNativeTokens,
        send_nft::AddressAndNftId,
    },
    error::Error,
};

/// The wallet Result type.
pub type Result<T> = std::result::Result<T, Error>;
