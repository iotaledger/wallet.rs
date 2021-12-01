// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{account::operations::syncing::SyncOptions, account_manager::AccountManager};

use tokio::time::sleep;

use std::{sync::atomic::Ordering, time::Duration};

/// The default interval for background syncing
pub(crate) const DEFAUTL_BACKGROUNDSYNCING_INTERVAL: Duration = Duration::from_secs(7);

/// Start the background syncing process for all accounts, default interval is 7 seconds
pub async fn start_background_syncing(
    account_manager: &AccountManager,
    options: Option<SyncOptions>,
    interval: Option<Duration>,
) -> crate::Result<()> {
    log::debug!("[start_background_syncing]");
    let background_syncing_status = account_manager.background_syncing_status.clone();
    // stop existing process if running
    if background_syncing_status.load(Ordering::Relaxed) == 1 {
        background_syncing_status.store(2, Ordering::Relaxed);
    };
    while background_syncing_status.load(Ordering::Relaxed) == 2 {
        log::debug!("[background_syncing]: waiting for the old process to stop");
        sleep(Duration::from_secs(1)).await;
    }

    background_syncing_status.store(1, Ordering::Relaxed);
    let accounts = account_manager.accounts.clone();
    let background_syncing = std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        runtime.block_on(async {
            'outer: loop {
                log::debug!("[background_syncing]: syncing accounts");
                let accounts = accounts.read().await;
                for account in accounts.iter() {
                    // Check if the process should stop before syncing each account so it stops faster
                    if background_syncing_status.load(Ordering::Relaxed) == 2 {
                        log::debug!("[background_syncing]: stopping");
                        break 'outer;
                    }
                    match account.sync(options.clone()).await {
                        Ok(_) => {}
                        Err(err) => log::debug!("[background_syncing] error: {}", err),
                    };
                }
                // split interval syncing to seconds so stopping the process doesn't have to wait long
                let seconds = interval.unwrap_or(DEFAUTL_BACKGROUNDSYNCING_INTERVAL).as_secs();
                for second in 0..seconds {
                    if background_syncing_status.load(Ordering::Relaxed) == 2 {
                        log::debug!("[background_syncing]: stopping");
                        break 'outer;
                    }
                    sleep(Duration::from_secs(1)).await;
                }
            }
            background_syncing_status.store(0, Ordering::Relaxed);
            log::debug!("[background_syncing]: stopped");
        });
    });
    Ok(())
}
