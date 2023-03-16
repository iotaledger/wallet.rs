// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::str::FromStr;

use clap::{Parser, Subcommand};
use iota_wallet::{
    account::{
        types::{AccountAddress, TransactionDto},
        AccountHandle, OutputsToClaim,
    },
    iota_client::{
        api_types::plugins::participation::types::ParticipationEventId,
        block::{
            address::Address,
            output::{
                unlock_condition::AddressUnlockCondition, AliasId, BasicOutputBuilder, FoundryId, NativeToken, NftId,
                OutputId, TokenId, UnlockCondition,
            },
        },
        request_funds_from_faucet,
    },
    AddressAndNftId, AddressNativeTokens, AddressWithAmount, AddressWithMicroAmount, NativeTokenOptions, NftOptions,
    U256,
};

use crate::error::Error;

#[derive(Debug, Parser)]
#[clap(version, long_about = None)]
#[clap(propagate_version = true)]
pub struct AccountCli {
    #[clap(subcommand)]
    pub command: AccountCommand,
}

#[derive(Debug, Subcommand)]
pub enum AccountCommand {
    /// List the account addresses.
    Addresses,
    /// Print the account balance.
    Balance,
    /// Burn a native token: `burn-native-token 0x... 100`
    BurnNativeToken { token_id: String, amount: String },
    /// Burn an NFT: `burn-nft 0x...`
    BurnNft { nft_id: String },
    /// Claim outputs with storage deposit return, expiration or timelock unlock conditions.
    Claim { output_id: Option<String> },
    /// Consolidate all basic outputs into one address.
    Consolidate,
    /// Create a new alias output.
    CreateAliasOutput,
    /// Melt a native token: `decrease-native-token-supply 0x... 100`
    DecreaseNativeTokenSupply { token_id: String, amount: String },
    /// Destroy an alias: `destroy-alias 0x...`
    DestroyAlias { alias_id: String },
    /// Destroy a foundry: `destroy-foundry 0x...`
    DestroyFoundry { foundry_id: String },
    /// Exit from the account prompt.
    Exit,
    /// Request funds from the faucet to the latest address, `url` is optional, default is `https://faucet.testnet.shimmer.network/api/enqueue`
    Faucet {
        url: Option<String>,
        address: Option<String>,
    },
    /// Mint more of a native token: `increase-native-token-supply 0x... 100`
    IncreaseNativeTokenSupply { token_id: String, amount: String },
    /// Mint a native token: `mint-native-token 100 100 --foundry-metadata-hex 0x...`
    MintNativeToken {
        circulating_supply: String,
        maximum_supply: String,
        #[clap(long, group = "foundry_metadata")]
        foundry_metadata_hex: Option<String>,
        #[clap(long, group = "foundry_metadata")]
        foundry_metadata_file: Option<String>,
    },
    /// Mint an NFT to an optional bech32 encoded address: `mint-nft
    /// rms1qztwng6cty8cfm42nzvq099ev7udhrnk0rw8jt8vttf9kpqnxhpsx869vr3 "immutable metadata" "metadata"`
    /// IOTA NFT Standard - TIP27: https://github.com/iotaledger/tips/blob/main/tips/TIP-0027/tip-0027.md
    MintNft {
        address: Option<String>,
        #[clap(long, group = "immutable_metadata")]
        immutable_metadata_hex: Option<String>,
        #[clap(long, group = "immutable_metadata")]
        immutable_metadata_file: Option<String>,
        #[clap(long, group = "metadata")]
        metadata_hex: Option<String>,
        #[clap(long, group = "metadata")]
        metadata_file: Option<String>,
        #[clap(long)]
        tag: Option<String>,
        #[clap(long)]
        sender: Option<String>,
        #[clap(long)]
        issuer: Option<String>,
    },
    /// Generate a new address.
    NewAddress,
    /// Display an output.
    Output { output_id: String },
    /// List all outputs.
    Outputs,
    /// Send an amount to a bech32 encoded address: `send
    /// rms1qztwng6cty8cfm42nzvq099ev7udhrnk0rw8jt8vttf9kpqnxhpsx869vr3 1000000`
    Send { address: String, amount: u64 },
    /// Send an amount below the storage deposit minimum to a bech32 address: `send
    /// rms1qztwng6cty8cfm42nzvq099ev7udhrnk0rw8jt8vttf9kpqnxhpsx869vr3 1`
    SendMicro { address: String, amount: u64 },
    /// Send native tokens to a bech32 address: `send-native-token
    /// rms1qztwng6cty8cfm42nzvq099ev7udhrnk0rw8jt8vttf9kpqnxhpsx869vr3
    /// 0x08e3a2f76cc934bc0cc21575b4610c1d7d4eb589ae0100000000000000000000000000000000 10`
    /// This will create an output with an expiration and storage deposit return unlock condition. To gift the storage
    /// deposit for the output, add ` true`.
    SendNativeToken {
        address: String,
        token_id: String,
        amount: String,
        gift_storage_deposit: Option<bool>,
    },
    /// Send an NFT to a bech32 encoded address
    SendNft { address: String, nft_id: String },
    /// Sync the account with the Tangle.
    Sync,
    /// List the account transactions.
    Transactions,
    /// List the unspent outputs.
    UnspentOutputs,
    /// Cast given votes for a given event
    Vote {
        event_id: ParticipationEventId,
        answers: Vec<u8>,
    },
    /// Stop participating to a given event
    StopParticipating { event_id: ParticipationEventId },
    /// Calculate the participation overview of the account
    ParticipationOverview {
        #[clap(short, long, value_parser, num_args = 1.., value_delimiter = ' ')]
        event_ids: Vec<ParticipationEventId>,
    },
    /// Get the voting power of the account
    VotingPower,
    /// Increase the voting power of the account
    IncreaseVotingPower { amount: u64 },
    /// Decrease the voting power of the account
    DecreaseVotingPower { amount: u64 },
    /// Get the voting output of the account
    VotingOutput,
}

/// `addresses` command
pub async fn addresses_command(account_handle: &AccountHandle) -> Result<(), Error> {
    let addresses = account_handle.addresses().await?;

    if addresses.is_empty() {
        log::info!("No addresses found");
    } else {
        for address in addresses {
            print_address(account_handle, &address).await?;
        }
    }

    Ok(())
}

// `burn-native-token` command
pub async fn burn_native_token_command(
    account_handle: &AccountHandle,
    token_id: String,
    amount: String,
) -> Result<(), Error> {
    log::info!("Burning native token {token_id} {amount}.");

    let transaction = account_handle
        .burn_native_token(
            TokenId::from_str(&token_id)?,
            U256::from_dec_str(&amount).map_err(|e| Error::Miscellaneous(e.to_string()))?,
            None,
        )
        .await?;

    log::info!(
        "Burning transaction sent:\n{:?}\n{:?}",
        transaction.transaction_id,
        transaction.block_id
    );

    Ok(())
}

// `burn-nft` command
pub async fn burn_nft_command(account_handle: &AccountHandle, nft_id: String) -> Result<(), Error> {
    log::info!("Burning nft {nft_id}.");

    let transaction = account_handle.burn_nft(NftId::from_str(&nft_id)?, None).await?;

    log::info!(
        "Burning transaction sent:\n{:?}\n{:?}",
        transaction.transaction_id,
        transaction.block_id
    );

    Ok(())
}

// `balance` command
pub async fn balance_command(account_handle: &AccountHandle) -> Result<(), Error> {
    log::info!("{:?}", account_handle.balance().await?);

    Ok(())
}

// `claim` command
pub async fn claim_command(account_handle: &AccountHandle, output_id: Option<String>) -> Result<(), Error> {
    if let Some(output_id) = output_id {
        log::info!("Claiming output {output_id}");

        let transaction = account_handle
            .claim_outputs(vec![OutputId::from_str(&output_id)?])
            .await?;

        log::info!(
            "Claiming transaction sent:\n{:?}\n{:?}",
            transaction.transaction_id,
            transaction.block_id
        );
    } else {
        log::info!("Claiming outputs.");

        let output_ids = account_handle
            .get_unlockable_outputs_with_additional_unlock_conditions(OutputsToClaim::All)
            .await?;

        if output_ids.is_empty() {
            log::info!("No outputs available to claim.");
        }

        // Doing chunks of only 60, because we might need to create the double amount of outputs, because of potential
        // storage deposit return unlock conditions and also consider the remainder output.
        for output_ids_chunk in output_ids.chunks(60) {
            let transaction = account_handle.claim_outputs(output_ids_chunk.to_vec()).await?;
            log::info!(
                "Claiming transaction sent:\n{:?}\n{:?}",
                transaction.transaction_id,
                transaction.block_id
            );
        }
    };

    Ok(())
}

// `consolidate` command
pub async fn consolidate_command(account_handle: &AccountHandle) -> Result<(), Error> {
    log::info!("Consolidating outputs.");

    let transaction = account_handle.consolidate_outputs(true, None).await?;

    log::info!(
        "Consolidation transaction sent:\n{:?}\n{:?}",
        transaction.transaction_id,
        transaction.block_id
    );

    Ok(())
}

// `create-alias-output` command
pub async fn create_alias_outputs_command(account_handle: &AccountHandle) -> Result<(), Error> {
    log::info!("Creating alias output.");

    let transaction = account_handle.create_alias_output(None, None).await?;

    log::info!(
        "Alias output creation transaction sent:\n{:?}\n{:?}",
        transaction.transaction_id,
        transaction.block_id
    );

    Ok(())
}

// `decrease-native-token-supply` command
pub async fn decrease_native_token_command(
    account_handle: &AccountHandle,
    token_id: String,
    amount: String,
) -> Result<(), Error> {
    let transaction = account_handle
        .decrease_native_token_supply(
            TokenId::from_str(&token_id)?,
            U256::from_dec_str(&amount).map_err(|e| Error::Miscellaneous(e.to_string()))?,
            None,
        )
        .await?;

    log::info!(
        "Native token melting transaction sent:\n{:?}\n{:?}",
        transaction.transaction_id,
        transaction.block_id
    );

    Ok(())
}

// `destroy-alias` command
pub async fn destroy_alias_command(account_handle: &AccountHandle, alias_id: String) -> Result<(), Error> {
    log::info!("Destroying alias {alias_id}.");

    let transaction = account_handle
        .destroy_alias(AliasId::from_str(&alias_id)?, None)
        .await?;

    log::info!(
        "Destroying alias transaction sent:\n{:?}\n{:?}",
        transaction.transaction_id,
        transaction.block_id
    );

    Ok(())
}

// `destroy-foundry` command
pub async fn destroy_foundry_command(account_handle: &AccountHandle, foundry_id: String) -> Result<(), Error> {
    log::info!("Destroying foundry {foundry_id}.");

    let transaction = account_handle
        .destroy_foundry(FoundryId::from_str(&foundry_id)?, None)
        .await?;

    log::info!(
        "Destroying foundry transaction sent:\n{:?}\n{:?}",
        transaction.transaction_id,
        transaction.block_id
    );

    Ok(())
}

// `faucet` command
pub async fn faucet_command(
    account_handle: &AccountHandle,
    url: Option<String>,
    address: Option<String>,
) -> Result<(), Error> {
    let address = if let Some(address) = address {
        address
    } else {
        match account_handle.addresses().await?.last() {
            Some(address) => address.address().to_bech32(),
            None => return Err(Error::NoAddressForFaucet),
        }
    };
    let faucet_url = match &url {
        Some(faucet_url) => faucet_url,
        None => "https://faucet.testnet.shimmer.network/api/enqueue",
    };

    log::info!("{}", request_funds_from_faucet(faucet_url, &address).await?);

    Ok(())
}

// `increase-native-token-supply` command
pub async fn increase_native_token_command(
    account_handle: &AccountHandle,
    token_id: String,
    amount: String,
) -> Result<(), Error> {
    let mint_transaction = account_handle
        .increase_native_token_supply(
            TokenId::from_str(&token_id)?,
            U256::from_dec_str(&amount).map_err(|e| Error::Miscellaneous(e.to_string()))?,
            None,
            None,
        )
        .await?;

    log::info!(
        "Minting more native token transaction sent:\n{:?}\n{:?}",
        mint_transaction.transaction.transaction_id,
        mint_transaction.transaction.block_id
    );

    Ok(())
}

// `mint-native-token` command
pub async fn mint_native_token_command(
    account_handle: &AccountHandle,
    circulating_supply: String,
    maximum_supply: String,
    foundry_metadata: Option<Vec<u8>>,
) -> Result<(), Error> {
    // If no alias output exists, create one first
    if account_handle.balance().await?.aliases.is_empty() {
        let transaction = account_handle.create_alias_output(None, None).await?;
        log::info!(
            "Alias output minting transaction sent:\n{:?}\n{:?}",
            transaction.transaction_id,
            transaction.block_id
        );
        account_handle
            .retry_transaction_until_included(&transaction.transaction_id, None, None)
            .await?;
        // Sync account after the transaction got confirmed, so the alias output is available
        account_handle.sync(None).await?;
    }

    let native_token_options = NativeTokenOptions {
        alias_id: None,
        circulating_supply: U256::from_dec_str(&circulating_supply).map_err(|e| Error::Miscellaneous(e.to_string()))?,
        maximum_supply: U256::from_dec_str(&maximum_supply).map_err(|e| Error::Miscellaneous(e.to_string()))?,
        foundry_metadata,
    };

    let mint_transaction = account_handle.mint_native_token(native_token_options, None).await?;

    log::info!(
        "Native token minting transaction sent:\n{:?}\n{:?}",
        mint_transaction.transaction.transaction_id,
        mint_transaction.transaction.block_id
    );

    Ok(())
}

// `mint-nft` command
pub async fn mint_nft_command(
    account_handle: &AccountHandle,
    address: Option<String>,
    immutable_metadata: Option<Vec<u8>>,
    metadata: Option<Vec<u8>>,
    tag: Option<String>,
    sender: Option<String>,
    issuer: Option<String>,
) -> Result<(), Error> {
    let tag = if let Some(hex) = tag {
        Some(prefix_hex::decode(&hex).map_err(|e| Error::Miscellaneous(e.to_string()))?)
    } else {
        None
    };
    let nft_options = vec![NftOptions {
        issuer,
        sender,
        tag,
        address,
        immutable_metadata,
        metadata,
    }];
    let transaction = account_handle.mint_nfts(nft_options, None).await?;

    log::info!(
        "NFT minting transaction sent:\n{:?}\n{:?}",
        transaction.transaction_id,
        transaction.block_id
    );

    Ok(())
}

// `new-address` command
pub async fn new_address_command(account_handle: &AccountHandle) -> Result<(), Error> {
    let address = account_handle.generate_addresses(1, None).await?;

    print_address(account_handle, &address[0]).await?;

    Ok(())
}

/// `output` command
pub async fn output_command(account_handle: &AccountHandle, output_id: String) -> Result<(), Error> {
    let output = account_handle.get_output(&OutputId::from_str(&output_id)?).await;

    if let Some(output) = output {
        log::info!("{output:#?}");
    } else {
        log::info!("Output not found");
    }

    Ok(())
}

/// `outputs` command
pub async fn outputs_command(account_handle: &AccountHandle) -> Result<(), Error> {
    let outputs = account_handle.outputs(None).await?;

    if outputs.is_empty() {
        log::info!("No outputs found");
    } else {
        let output_ids: Vec<OutputId> = outputs.iter().map(|o| o.output_id).collect();
        log::info!("Outputs: {output_ids:#?}");
    }

    Ok(())
}

// `send` command
pub async fn send_command(account_handle: &AccountHandle, address: String, amount: u64) -> Result<(), Error> {
    let outputs = vec![AddressWithAmount { address, amount }];
    let transaction = account_handle.send_amount(outputs, None).await?;

    log::info!(
        "Transaction sent:\n{:?}\n{:?}",
        transaction.transaction_id,
        transaction.block_id
    );

    Ok(())
}

// `send-micro` command
pub async fn send_micro_command(account_handle: &AccountHandle, address: String, amount: u64) -> Result<(), Error> {
    let outputs = vec![AddressWithMicroAmount {
        address,
        amount,
        return_address: None,
        expiration: None,
    }];

    let transaction = account_handle.send_micro_transaction(outputs, None).await?;

    log::info!(
        "Micro transaction sent:\n{:?}\n{:?}",
        transaction.transaction_id,
        transaction.block_id
    );

    Ok(())
}

// `send-native-token` command
pub async fn send_native_token_command(
    account_handle: &AccountHandle,
    address: String,
    token_id: String,
    amount: String,
    gift_storage_deposit: Option<bool>,
) -> Result<(), Error> {
    let transaction = if gift_storage_deposit.unwrap_or(false) {
        // Send native tokens together with the required storage deposit
        let rent_structure = account_handle.client().get_rent_structure().await?;
        let token_supply = account_handle.client().get_token_supply().await?;

        let outputs = vec![
            BasicOutputBuilder::new_with_minimum_storage_deposit(rent_structure)?
                .add_unlock_condition(UnlockCondition::Address(AddressUnlockCondition::new(
                    Address::try_from_bech32(address)?.1,
                )))
                .with_native_tokens(vec![NativeToken::new(
                    TokenId::from_str(&token_id)?,
                    U256::from_dec_str(&amount).map_err(|e| Error::Miscellaneous(e.to_string()))?,
                )?])
                .finish_output(token_supply)?,
        ];

        account_handle.send(outputs, None).await?
    } else {
        // Send native tokens with storage deposit return and expiration
        let outputs = vec![AddressNativeTokens {
            address,
            native_tokens: vec![(
                TokenId::from_str(&token_id)?,
                U256::from_dec_str(&amount).map_err(|e| Error::Miscellaneous(e.to_string()))?,
            )],
            ..Default::default()
        }];
        account_handle.send_native_tokens(outputs, None).await?
    };

    log::info!(
        "Native token transaction sent:\n{:?}\n{:?}",
        transaction.transaction_id,
        transaction.block_id
    );

    Ok(())
}

// `send-nft` command
pub async fn send_nft_command(account_handle: &AccountHandle, address: String, nft_id: String) -> Result<(), Error> {
    let outputs = vec![AddressAndNftId {
        address,
        nft_id: NftId::from_str(&nft_id)?,
    }];
    let transaction = account_handle.send_nft(outputs, None).await?;

    log::info!(
        "Nft transaction sent:\n{:?}\n{:?}",
        transaction.transaction_id,
        transaction.block_id
    );

    Ok(())
}

// `sync` command
pub async fn sync_command(account_handle: &AccountHandle) -> Result<(), Error> {
    let sync = account_handle.sync(None).await?;

    log::info!("Synced: {sync:?}");

    Ok(())
}

/// `transactions` command
pub async fn transactions_command(account_handle: &AccountHandle) -> Result<(), Error> {
    let transactions = account_handle.transactions().await?;

    if transactions.is_empty() {
        log::info!("No transactions found");
    } else {
        for tx in transactions {
            log::info!("{}", serde_json::to_string(&TransactionDto::from(&tx))?);
        }
    }

    Ok(())
}

/// `unspent-outputs` command
pub async fn unspent_outputs_command(account_handle: &AccountHandle) -> Result<(), Error> {
    let outputs = account_handle.unspent_outputs(None).await?;

    if outputs.is_empty() {
        log::info!("No outputs found");
    } else {
        let output_ids: Vec<OutputId> = outputs.iter().map(|o| o.output_id).collect();
        log::info!("Unspent outputs: {output_ids:#?}");
    }

    Ok(())
}

pub async fn vote_command(
    account_handle: &AccountHandle,
    event_id: ParticipationEventId,
    answers: Vec<u8>,
) -> Result<(), Error> {
    let transaction = account_handle.vote(Some(event_id), Some(answers)).await?;

    log::info!(
        "Voting transaction sent:\n{:?}\n{:?}",
        transaction.transaction_id,
        transaction.block_id
    );

    Ok(())
}

pub async fn stop_participating_command(
    account_handle: &AccountHandle,
    event_id: ParticipationEventId,
) -> Result<(), Error> {
    let transaction = account_handle.stop_participating(event_id).await?;

    log::info!(
        "Stop participating transaction sent:\n{:?}\n{:?}",
        transaction.transaction_id,
        transaction.block_id
    );

    Ok(())
}

pub async fn participation_overview_command(
    account_handle: &AccountHandle,
    event_ids: Option<Vec<ParticipationEventId>>,
) -> Result<(), Error> {
    let participation_overview = account_handle.get_participation_overview(event_ids).await?;

    log::info!("Participation overview: {participation_overview:?}");

    Ok(())
}

pub async fn voting_power_command(account_handle: &AccountHandle) -> Result<(), Error> {
    let voting_power = account_handle.get_voting_power().await?;

    log::info!("Voting power: {voting_power}");

    Ok(())
}

pub async fn increase_voting_power_command(account_handle: &AccountHandle, amount: u64) -> Result<(), Error> {
    let transaction = account_handle.increase_voting_power(amount).await?;

    log::info!(
        "Increase voting power transaction sent:\n{:?}\n{:?}",
        transaction.transaction_id,
        transaction.block_id
    );

    Ok(())
}

pub async fn decrease_voting_power_command(account_handle: &AccountHandle, amount: u64) -> Result<(), Error> {
    let transaction = account_handle.decrease_voting_power(amount).await?;

    log::info!(
        "Decrease voting power transaction sent:\n{:?}\n{:?}",
        transaction.transaction_id,
        transaction.block_id
    );

    Ok(())
}

pub async fn voting_output_command(account_handle: &AccountHandle) -> Result<(), Error> {
    let output = account_handle.get_voting_output().await?;

    log::info!("Voting output: {output:?}");

    Ok(())
}

async fn print_address(account_handle: &AccountHandle, address: &AccountAddress) -> Result<(), Error> {
    let mut log = format!("Address {}: {}", address.key_index(), address.address().to_bech32());

    if *address.internal() {
        log = format!("{log}\nChange address");
    }

    let addresses = account_handle.addresses_with_unspent_outputs().await?;

    if let Ok(index) = addresses.binary_search_by_key(&(address.key_index(), address.internal()), |a| {
        (a.key_index(), a.internal())
    }) {
        log = format!("{log}\nOutputs: {:#?}", addresses[index].output_ids());
    }

    log::info!("{log}");

    Ok(())
}
