use iota_wallet::{
    account_manager::AccountManager, client::ClientOptionsBuilder, message::MessageType,
};

#[tokio::main]
async fn main() -> iota_wallet::Result<()> {
    let manager = AccountManager::new().unwrap();
    manager.set_stronghold_password("password").unwrap();

    // first we'll create an example account and store it
    let client_options = ClientOptionsBuilder::node("https://nodes.devnet.iota.org:443")?.build();
    let mut account = manager
        .create_account(client_options)
        .alias("alias")
        .initialise()?;

    // update alias
    manager.set_alias(account.id().into(), "the new alias")?;
    // list unspent addresses
    let _ = account.list_addresses(false);
    // list spent addresses
    let _ = account.list_addresses(true);

    // generate a new unused address
    let _ = account.generate_address().await?;

    // list messages
    let _ = account.list_messages(5, 0, Some(MessageType::Failed));

    Ok(())
}
