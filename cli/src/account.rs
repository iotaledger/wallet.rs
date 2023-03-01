// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use dialoguer::Input;
use iota_wallet::account::AccountHandle;

use crate::{
    command::account::{
        addresses_command, balance_command, burn_native_token_command, burn_nft_command, claim_command,
        consolidate_command, create_alias_outputs_command, decrease_native_token_command,
        decrease_voting_power_command, destroy_alias_command, destroy_foundry_command, faucet_command,
        increase_native_token_command, increase_voting_power_command, mint_native_token_command, mint_nft_command,
        new_address_command, output_command, outputs_command, participation_overview_command, send_command,
        send_micro_command, send_native_token_command, send_nft_command, stop_participating_command, sync_command,
        transactions_command, unspent_outputs_command, vote_command, voting_output_command, voting_power_command,
        AccountCli, AccountCommand,
    },
    error::Error,
    helper::bytes_from_hex_or_file,
};

// loop on the account prompt
pub async fn account_prompt(account_handle: AccountHandle) -> Result<(), Error> {
    loop {
        match account_prompt_internal(account_handle.clone()).await {
            Ok(true) => {
                return Ok(());
            }
            Err(e) => {
                log::error!("{e}");
            }
            _ => {}
        }
    }
}

// loop on the account prompt
pub async fn account_prompt_internal(account_handle: AccountHandle) -> Result<bool, Error> {
    let alias = {
        let account = account_handle.read().await;
        account.alias().clone()
    };
    let command: String = Input::new().with_prompt(format!("Account \"{alias}\"")).interact()?;

    match command.as_str() {
        "h" => {
            if let Err(err) = AccountCli::try_parse_from(vec!["Account:", "help"]) {
                println!("{err}");
            }
        }
        "clear" => {
            // Clear console
            let _ = std::process::Command::new("clear").status();
        }
        _ => {
            // Prepend `Account: ` so the parsing will be correct
            let command = format!("Account: {}", command.trim());
            let account_cli = match AccountCli::try_parse_from(command.split(' ')) {
                Ok(account_cli) => account_cli,
                Err(err) => {
                    println!("{err}");
                    return Ok(false);
                }
            };
            if let Err(err) = match account_cli.command {
                AccountCommand::Addresses => addresses_command(&account_handle).await,
                AccountCommand::Balance => balance_command(&account_handle).await,
                AccountCommand::BurnNativeToken { token_id, amount } => {
                    burn_native_token_command(&account_handle, token_id, amount).await
                }
                AccountCommand::BurnNft { nft_id } => burn_nft_command(&account_handle, nft_id).await,
                AccountCommand::Claim { output_id } => claim_command(&account_handle, output_id).await,
                AccountCommand::Consolidate => consolidate_command(&account_handle).await,
                AccountCommand::CreateAliasOutput => create_alias_outputs_command(&account_handle).await,
                AccountCommand::DecreaseNativeTokenSupply { token_id, amount } => {
                    decrease_native_token_command(&account_handle, token_id, amount).await
                }
                AccountCommand::DestroyAlias { alias_id } => destroy_alias_command(&account_handle, alias_id).await,
                AccountCommand::DestroyFoundry { foundry_id } => {
                    destroy_foundry_command(&account_handle, foundry_id).await
                }
                AccountCommand::Exit => {
                    return Ok(true);
                }
                AccountCommand::Faucet { url, address } => faucet_command(&account_handle, url, address).await,
                AccountCommand::IncreaseNativeTokenSupply { token_id, amount } => {
                    increase_native_token_command(&account_handle, token_id, amount).await
                }
                AccountCommand::MintNativeToken {
                    circulating_supply,
                    maximum_supply,
                    foundry_metadata_hex,
                    foundry_metadata_file,
                } => {
                    mint_native_token_command(
                        &account_handle,
                        circulating_supply,
                        maximum_supply,
                        bytes_from_hex_or_file(foundry_metadata_hex, foundry_metadata_file).await?,
                    )
                    .await
                }
                AccountCommand::MintNft {
                    address,
                    immutable_metadata_hex,
                    immutable_metadata_file,
                    metadata_hex,
                    metadata_file,
                    tag,
                    sender,
                    issuer,
                } => {
                    mint_nft_command(
                        &account_handle,
                        address,
                        bytes_from_hex_or_file(immutable_metadata_hex, immutable_metadata_file).await?,
                        bytes_from_hex_or_file(metadata_hex, metadata_file).await?,
                        tag,
                        sender,
                        issuer,
                    )
                    .await
                }
                AccountCommand::NewAddress => new_address_command(&account_handle).await,
                AccountCommand::Output { output_id } => output_command(&account_handle, output_id).await,
                AccountCommand::Outputs => outputs_command(&account_handle).await,
                AccountCommand::Send { address, amount } => send_command(&account_handle, address, amount).await,
                AccountCommand::SendMicro { address, amount } => {
                    send_micro_command(&account_handle, address, amount).await
                }
                AccountCommand::SendNativeToken {
                    address,
                    token_id,
                    amount,
                    gift_storage_deposit,
                } => send_native_token_command(&account_handle, address, token_id, amount, gift_storage_deposit).await,
                AccountCommand::SendNft { address, nft_id } => send_nft_command(&account_handle, address, nft_id).await,
                AccountCommand::Sync => sync_command(&account_handle).await,
                AccountCommand::Transactions => transactions_command(&account_handle).await,
                AccountCommand::UnspentOutputs => unspent_outputs_command(&account_handle).await,
                AccountCommand::Vote { event_id, answers } => vote_command(&account_handle, event_id, answers).await,
                AccountCommand::StopParticipating { event_id } => {
                    stop_participating_command(&account_handle, event_id).await
                }
                AccountCommand::ParticipationOverview { event_id } => {
                    participation_overview_command(&account_handle, event_id).await
                }
                AccountCommand::VotingPower => voting_power_command(&account_handle).await,
                AccountCommand::IncreaseVotingPower { amount } => {
                    increase_voting_power_command(&account_handle, amount).await
                }
                AccountCommand::DecreaseVotingPower { amount } => {
                    decrease_voting_power_command(&account_handle, amount).await
                }
                AccountCommand::VotingOutput => voting_output_command(&account_handle).await,
            } {
                log::error!("{}", err);
            }
        }
    }

    Ok(false)
}
