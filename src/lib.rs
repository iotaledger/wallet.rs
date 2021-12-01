// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! The IOTA Wallet Library
#![allow(dead_code)]
#![allow(unused_variables)]

/// [`AccountHandle`]: crate::account::handle::AccountHandle
/// The account module. Interaction with an Account happens via an [`AccountHandle`].
pub mod account;
/// The account manager module.
pub mod account_manager;
/// The actor interface for the library. A different way to call the wallet functions, useful for bindings to other
/// languages.
// #[cfg(feature = "actor")]
// pub mod actor;

/// The client module to use iota_client for interactions with the IOTA Tangle.
pub mod client;
/// The error module.
pub mod error;
#[cfg(feature = "events")]
/// The event module.
pub mod events;
/// Signing interfaces for address generation and transaction signing.
pub mod signing;

// todo: add Stronghold support, refactor, check if it can't lock funds if it's stored with locked outputs and
// transaction creation failed so they should be unlocked again and other edge cases
#[cfg(feature = "storage")]
/// The storage module.
pub(crate) mod storage;

/// Module for debug logs.
pub mod logger;

// #[cfg(feature = "stronghold")]
// #[cfg_attr(docsrs, doc(cfg(feature = "stronghold")))]
// pub(crate) mod stronghold;

pub use error::Error;
/// The wallet Result type.
pub type Result<T> = std::result::Result<T, Error>;
