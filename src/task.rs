// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#[cfg(not(target_family = "wasm"))]
#[inline(always)]
pub(crate) fn spawn<F>(future: F) -> tokio::task::JoinHandle<F::Output>
where
    F: futures::Future + Send + 'static,
    F::Output: Send + 'static,
{
    tokio::task::spawn(future)
}

#[cfg(target_family = "wasm")]
#[inline(always)]
pub(crate) async fn spawn<F>(future: F) -> crate::Result<F::Output>
where
    F: futures::Future + 'static,
    F::Output: 'static,
{
    Ok(future.await)
}
