use iota_wallet::{
  account_manager::AccountManager, client::ClientOptionsBuilder, transaction::TransactionType,
};

#[tokio::main]
async fn main() -> iota_wallet::Result<()> {
  let manager = AccountManager::new();

  // first we'll create an example account and store it
  let id = "test";
  let client_options = ClientOptionsBuilder::node("https://nodes.devnet.iota.org:443")?.build();
  let mut account = manager
    .create_account(client_options)
    .alias(id)
    .id(id)
    .mnemonic(id)
    .initialise()?;

  // update alias
  account.set_alias("the new alias")?;
  // list unspent addresses
  let _ = account.list_addresses(false);
  // list spent addresses
  let _ = account.list_addresses(true);

  // generate a new unused address
  let _ = account.generate_address().await?;

  // list transactions
  let _ = account.list_transactions(5, 0, Some(TransactionType::Failed));

  Ok(())
}
