// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod account;
mod account_history;
mod account_manager;
mod command;
mod error;
mod helper;

use clap::Parser;
use fern_logger::{LoggerConfigBuilder, LoggerOutputConfigBuilder};
use log::LevelFilter;

use self::{
    account_manager::new_account_manager, command::account_manager::AccountManagerCli, error::Error,
    helper::pick_account,
};

fn logger_init(cli: &AccountManagerCli) -> Result<(), Error> {
    let stdout_level_filter = if let Some(log_level) = cli.log_level {
        log_level
    } else {
        LevelFilter::Info
    };
    let stdout = LoggerOutputConfigBuilder::default()
        .name("stdout")
        .level_filter(stdout_level_filter)
        .target_exclusions(&["rustls"])
        .color_enabled(true);
    let archive = LoggerOutputConfigBuilder::default()
        .name("archive.log")
        .level_filter(LevelFilter::Debug)
        .target_exclusions(&["rustls"])
        .color_enabled(false);
    let config = LoggerConfigBuilder::default()
        .with_output(stdout)
        .with_output(archive)
        .finish();

    fern_logger::logger_init(config)?;

    Ok(())
}

async fn run(cli: AccountManagerCli) -> Result<(), Error> {
    let (account_manager, account) = new_account_manager(cli.clone()).await?;

    if let Some(account_manager) = account_manager {
        match cli.account.or(account) {
            Some(account) => account::account_prompt(account_manager.get_account(account).await?).await?,
            None => {
                if let Some(account) = pick_account(&account_manager).await? {
                    account::account_prompt(account_manager.get_account(account).await?).await?;
                }
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    let cli = match AccountManagerCli::try_parse() {
        Ok(cli) => cli,
        Err(e) => {
            println!("{e}");
            return;
        }
    };

    if let Err(e) = logger_init(&cli) {
        println!("{e}");
        return;
    }

    if let Err(e) = run(cli).await {
        log::error!("{e}");
    }
}
