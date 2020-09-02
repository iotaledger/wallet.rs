use iota_wallet::{
    account_manager::AccountManager, client::ClientOptionsBuilder, transaction::Transfer,
};

#[tokio::main]
async fn main() -> iota_wallet::Result<()> {
    let manager = AccountManager::new();

    // first we'll create an example account and store it
    let client_options = ClientOptionsBuilder::node("https://nodes.devnet.iota.org:443")?.build();
    let account = manager
        .create_account(client_options)
        .alias("alias")
        .initialise()?;

    // we need to synchronize with the Tangle first
    let sync_accounts = manager.sync_accounts().await?;
    let sync_account = sync_accounts.first().unwrap();
    sync_account
        .transfer(Transfer::new(account.latest_address().clone(), 150))
        .await?;

    Ok(())
}
