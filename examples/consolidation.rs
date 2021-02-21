use iota::{
    api::finish_pow, bee_rest_api::handlers::message_metadata::LedgerInclusionStateDto, Client, ClientBuilder, Essence,
    Input, MessageId, Payload, RegularEssence, SignatureLockedDustAllowanceOutput, SignatureLockedSingleOutput,
    TransactionPayload, UTXOInput,
};
use iota_wallet::{
    account_manager::{AccountManager, ManagerStorage},
    client::ClientOptionsBuilder,
    signing::{get_signer, SignMessageMetadata, SignerType, TransactionInput},
    Result,
};
use serde::Deserialize;
use slip10::BIP32Path;

use std::{fs, str::FromStr, time::Duration};

#[derive(Deserialize)]
struct FaucetMessageResponse {
    id: String,
}

#[derive(Deserialize)]
struct FaucetResponse {
    data: FaucetMessageResponse,
}

#[tokio::main]
async fn main() -> Result<()> {
    let node_url = "https://api.lb-0.testnet.chrysalis2.com";
    let storage_path = "./storage/consolidation";

    // clear old storage so we can always start fresh
    let _ = fs::remove_dir_all(storage_path);

    // setup the account manager
    let mut manager = AccountManager::builder()
        .with_storage(storage_path, ManagerStorage::Stronghold, None)?
        .finish()
        .await?;
    manager.set_stronghold_password("password").await?;
    manager.store_mnemonic(SignerType::Stronghold, None).await?;

    // create an account
    let client_options = ClientOptionsBuilder::new().with_node(node_url)?.build()?;
    let account = manager
        .create_account(client_options)?
        .alias("alias")
        .initialise()
        .await?;

    // get the address we're going to use
    let address = account.read().await.addresses().first().unwrap().clone();
    println!("Address {}", address.address().to_bech32());

    // use the faucet to get funds on the address
    let response = reqwest::get(&format!(
        "https://faucet.testnet.chrysalis2.com/api?address={}",
        address.address().to_bech32()
    ))
    .await
    .unwrap()
    .json::<FaucetResponse>()
    .await
    .unwrap();
    let faucet_message_id = MessageId::from_str(&response.data.id)?;
    println!("Got funds from faucet, message id: {:?}", faucet_message_id);

    // create a iota.rs client to use the node API
    let iota_client = ClientBuilder::new()
        .with_node(node_url)
        .unwrap()
        .finish()
        .await
        .unwrap();

    // wait for the faucet transaction to be confirmed
    wait_for_message_confirmation(&iota_client, faucet_message_id).await?;
    println!("Faucet message confirmed");

    // force account update
    account.sync().await.execute().await?;

    // reload the address after sync
    let address = account.read().await.addresses().first().unwrap().clone();

    // add a dust allowance output on our address
    let output = address.outputs().first().unwrap();

    let input: Input = UTXOInput::new(output.transaction_id, output.index)?.into();
    let essence_builder = RegularEssence::builder()
        .add_output(SignatureLockedDustAllowanceOutput::new(*address.address().as_ref(), 1_000_000)?.into())
        .add_output(SignatureLockedSingleOutput::new(*address.address().as_ref(), 9_000_000)?.into())
        .add_input(input.clone());

    let essence = essence_builder.finish()?;
    let essence = Essence::Regular(essence);

    let unlock_blocks = get_signer(account.read().await.signer_type())
        .await
        .lock()
        .await
        .sign_message(
            &account.read().await.clone(),
            &essence,
            &mut vec![TransactionInput {
                input,
                address_index: *address.key_index(),
                address_path: BIP32Path::from_str(&format!(
                    "m/44H/4218H/{}H/{}H/{}H",
                    account.index().await,
                    *address.internal() as u32,
                    *address.key_index()
                ))
                .unwrap(),
                address_internal: *address.internal(),
            }],
            SignMessageMetadata {
                remainder_address: None,
                remainder_value: 0,
                remainder_deposit_address: None,
            },
        )
        .await?;

    let mut tx_builder = TransactionPayload::builder().with_essence(essence);
    for unlock_block in unlock_blocks {
        tx_builder = tx_builder.add_unlock_block(unlock_block);
    }
    let transaction = tx_builder.finish()?;

    let message = finish_pow(&iota_client, Some(Payload::Transaction(Box::new(transaction)))).await?;
    let message_id = iota_client.post_message(&message).await?;

    println!("Dust allowance output message id: {:?}", message_id);
    wait_for_message_confirmation(&iota_client, message_id).await?;
    println!("Dust allowance output message confirmed");

    // force account update
    account.sync().await.execute().await?;

    // reload the address after sync
    let address = account.read().await.addresses().first().unwrap().clone();
    println!("{:?}", address);

    Ok(())
}

async fn wait_for_message_confirmation(client: &Client, message_id: MessageId) -> Result<()> {
    loop {
        let metadata = client.get_message().metadata(&message_id).await?;
        if let Some(state) = &metadata.ledger_inclusion_state {
            if state == &LedgerInclusionStateDto::Included {
                break;
            } else {
                panic!(format!("faucet message wasn't confirmed; {:?}", metadata));
            }
        } else {
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    }
    Ok(())
}
