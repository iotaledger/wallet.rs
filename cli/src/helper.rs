// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use dialoguer::{console::Term, theme::ColorfulTheme, Password, Select};
use iota_wallet::account_manager::AccountManager;

use crate::error::Error;

pub fn get_password(prompt: &str, confirmation: bool) -> Result<String, Error> {
    let mut password = Password::new();

    password.with_prompt(prompt);

    if confirmation {
        password.with_confirmation("Confirm password", "Password mismatch");
    }

    Ok(password.interact()?)
}

pub async fn pick_account(manager: &AccountManager) -> Result<Option<u32>, Error> {
    let accounts = manager.get_accounts().await?;

    match accounts.len() {
        0 => Ok(None),
        1 => Ok(Some(0)),
        _ => {
            let mut items = Vec::new();

            for account_handle in accounts {
                items.push(account_handle.read().await.alias().clone());
            }

            let index = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select an account:")
                .items(&items)
                .default(0)
                .interact_on(&Term::stderr())?;

            Ok(Some(index as u32))
        }
    }
}

pub async fn bytes_from_hex_or_file(hex: Option<String>, file: Option<String>) -> Result<Option<Vec<u8>>, Error> {
    Ok(if let Some(hex) = hex {
        Some(prefix_hex::decode(&hex).map_err(|e| Error::Miscellaneous(e.to_string()))?)
    } else if let Some(file) = file {
        Some(tokio::fs::read(file).await?)
    } else {
        None
    })
}
