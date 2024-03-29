// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! The IOTA Wallet Library

#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(clippy::nursery, rust_2018_idioms, warnings, unreachable_pub)]
#![allow(
    clippy::redundant_pub_crate,
    clippy::module_name_repetitions,
    clippy::missing_const_for_fn,
    clippy::significant_drop_in_scrutinee
)]

/// [`AccountHandle`]: crate::account::handle::AccountHandle
/// The account module. Interaction with an Account happens via an [`AccountHandle`].
pub mod account;
/// The account manager module.
pub mod account_manager;
/// The message passing interface for the library. A different way to call the wallet functions, useful for bindings to
/// other languages.
#[cfg(feature = "message_interface")]
#[cfg_attr(docsrs, doc(cfg(feature = "message_interface")))]
pub mod message_interface;

/// The ClientOptions to build the iota_client for interactions with the IOTA Tangle.
pub use iota_client::ClientBuilder as ClientOptions;

/// The error module.
pub mod error;
/// The event module.
#[cfg(feature = "events")]
#[cfg_attr(docsrs, doc(cfg(feature = "events")))]
pub mod events;
/// The storage module.
#[cfg(feature = "storage")]
#[cfg_attr(docsrs, doc(cfg(feature = "storage")))]
pub mod storage;
/// The module for spawning tasks on a thread
pub(crate) mod task;

// Expose for high level functions
pub use iota_client::{self, secret};
pub use primitive_types::U256;

pub use self::{
    account::operations::transaction::high_level::{
        minting::{
            increase_native_token_supply::IncreaseNativeTokenSupplyOptions, mint_native_token::NativeTokenOptions,
            mint_nfts::NftOptions,
        },
        send_amount::AddressWithAmount,
        send_micro_transaction::AddressWithMicroAmount,
        send_native_tokens::AddressNativeTokens,
        send_nft::AddressAndNftId,
    },
    error::Error,
};

/// The wallet Result type.
pub type Result<T> = std::result::Result<T, Error>;
