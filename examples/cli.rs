//! Wallet CLI example
//! Create a new account: `$ cargo run --example cli -- new --node http://localhost:14265`

use clap::{load_yaml, App, AppSettings, ArgMatches};
use console::Term;
use dialoguer::{theme::ColorfulTheme, Input, Select};
use iota::message::prelude::MessageId;
use iota_wallet::{
  account::Account,
  account_manager::AccountManager,
  client::ClientOptionsBuilder,
  message::{Message, MessageType, Transfer},
  Result,
};
use once_cell::sync::OnceCell;
use tokio::runtime::Runtime;

use std::{str::FromStr, sync::Mutex};

const CLI_TEMPLATE: &'static str = "\
  {before-help}{bin} {version}\n\
  {about-with-newline}\n\
  {usage-heading}\n    {usage}\n\
  \n\
  {all-args}{after-help}\
";

const ACCOUNT_CLI_TEMPLATE: &'static str = "\
  {all-args}{after-help}\
";

fn print_message(message: &Message) {
  println!("{:?}", message);
}

static RUNTIME: OnceCell<Mutex<Runtime>> = OnceCell::new();

fn block_on<C: futures::Future>(cb: C) -> C::Output {
  let runtime = RUNTIME.get().unwrap();
  runtime.lock().unwrap().block_on(cb)
}

fn list_messages_command(account: &Account, matches: &ArgMatches) {
  if let Some(matches) = matches.subcommand_matches("list-messages") {
    if let Some(id) = matches.value_of("id") {
      if let Ok(message_id) = MessageId::from_str(id) {
        if let Some(message) = account.get_message(&message_id) {
          print_message(message);
        } else {
          println!("Message not found");
        }
      } else {
        println!("Message id must be a hex string of length 64");
      }
    } else {
      let message_type = if let Some(message_type) = matches.value_of("type") {
        match message_type {
          "received" => Some(MessageType::Received),
          "sent" => Some(MessageType::Sent),
          "failed" => Some(MessageType::Failed),
          "unconfirmed" => Some(MessageType::Unconfirmed),
          "value" => Some(MessageType::Value),
          _ => panic!("unexpected message type"),
        }
      } else {
        None
      };

      let messages = account.list_messages(0, 0, message_type);
      if messages.is_empty() {
        println!("No messages found");
      } else {
        messages.iter().for_each(|m| println!("{:?}\n\n", m));
      }
    }
  }
}

fn list_addresses_command(account: &Account, matches: &ArgMatches) {
  if let Some(_) = matches.subcommand_matches("list-addresses") {
    let mut addresses = account.list_addresses(false);
    addresses.extend(account.list_addresses(true));
    if addresses.is_empty() {
      println!("No addresses found");
    } else {
      addresses.iter().for_each(|m| println!("{:?}\n\n", m));
    }
  }
}

fn sync_account_command(account: &mut Account, matches: &ArgMatches) -> Result<()> {
  if matches.subcommand_matches("sync").is_some() {
    block_on(async move { account.sync().execute().await })?;
  }
  Ok(())
}

fn generate_address_command(account: &mut Account, matches: &ArgMatches) -> Result<()> {
  if let Some(_) = matches.subcommand_matches("address") {
    let address = account.generate_address()?;
    println!("{}", address.address().to_bech32());
  }
  Ok(())
}

fn balance_command(account: &Account, matches: &ArgMatches) {
  if let Some(_) = matches.subcommand_matches("balance") {
    let balance = account
      .addresses()
      .iter()
      .fold(0, |acc, addr| acc + *addr.balance());
    println!("{}", balance);
  }
}

enum ReplayAction {
  Promote,
  Retry,
  Reattach,
}

fn replay_message(account: &mut Account, action: ReplayAction, message_id: &str) -> Result<()> {
  if let Ok(message_id) = MessageId::from_str(message_id) {
    let res: Result<()> = block_on(async move {
      let synced = account.sync().execute().await?;
      let message = match action {
        ReplayAction::Promote => synced.promote(&message_id).await?,
        ReplayAction::Retry => synced.retry(&message_id).await?,
        ReplayAction::Reattach => synced.reattach(&message_id).await?,
      };
      print_message(&message);
      Ok(())
    });
    res?;
  } else {
    println!("Message id must be a hex string of length 64");
  }
  Ok(())
}

fn transfer_command(account: &mut Account, matches: &ArgMatches) -> Result<()> {
  if let Some(matches) = matches.subcommand_matches("transfer") {
    let address = matches.value_of("address").unwrap().to_string();
    let amount = matches.value_of("amount").unwrap();
    if let Ok(address) = iota_wallet::address::parse(address) {
      if let Ok(amount) = amount.parse::<u64>() {
        let transfer = Transfer::new(address, amount);
        let res: Result<()> = block_on(async move {
          let synced = account.sync().execute().await?;
          let message = synced.transfer(transfer).await?;
          print_message(&message);
          Ok(())
        });
        res?;
      } else {
        println!("Amount must be a number");
      }
    } else {
      println!("Address must be a bech32 string");
    }
  }
  Ok(())
}

fn promote_message_command(account: &mut Account, matches: &ArgMatches) -> Result<()> {
  if let Some(matches) = matches.subcommand_matches("promote") {
    let message_id = matches.value_of("id").unwrap();
    replay_message(account, ReplayAction::Promote, message_id)?;
  }
  Ok(())
}

fn retry_message_command(account: &mut Account, matches: &ArgMatches) -> Result<()> {
  if let Some(matches) = matches.subcommand_matches("retry") {
    let message_id = matches.value_of("id").unwrap();
    replay_message(account, ReplayAction::Retry, message_id)?;
  }
  Ok(())
}

fn reattach_message_command(account: &mut Account, matches: &ArgMatches) -> Result<()> {
  if let Some(matches) = matches.subcommand_matches("reattach") {
    let message_id = matches.value_of("id").unwrap();
    replay_message(account, ReplayAction::Reattach, message_id)?;
  }
  Ok(())
}

fn account_commands(account: &mut Account, matches: &ArgMatches) -> Result<()> {
  list_messages_command(account, &matches);
  list_addresses_command(account, &matches);
  sync_account_command(account, &matches)?;
  generate_address_command(account, &matches)?;
  balance_command(account, &matches);
  transfer_command(account, &matches)?;
  promote_message_command(account, &matches)?;
  retry_message_command(account, &matches)?;
  reattach_message_command(account, &matches)?;
  Ok(())
}

fn enter_account(account_cli: &App<'_>, mut account: Account) {
  let command: String = Input::new()
    .with_prompt(format!(
      "Account `{}` command (h for help)",
      account.alias()
    ))
    .interact_text()
    .unwrap();

  match command.as_str() {
    "h" => {
      let mut cli = account_cli.clone();
      cli.print_help().unwrap();
    }
    "clear" => {
      let _ = std::process::Command::new("clear").status();
    }
    _ => {
      match account_cli
        .clone()
        .try_get_matches_from(command.split(' ').collect::<Vec<&str>>())
      {
        Ok(matches) => {
          if matches.subcommand_matches("exit").is_some() {
            return;
          }

          if let Err(e) = account_commands(&mut account, &matches) {
            println!("{:?}", e);
          }
        }
        Err(e) => {
          println!("{}", e.to_string());
        }
      }
    }
  }

  enter_account(account_cli, account)
}

fn pick_account(account_cli: &App<'_>, accounts: Vec<Account>) -> Result<()> {
  let items: Vec<&String> = accounts.iter().map(|acc| acc.alias()).collect();
  let selection = Select::with_theme(&ColorfulTheme::default())
    .with_prompt("Select an account to manipulate")
    .items(&items)
    .default(0)
    .interact_on_opt(&Term::stderr())?;
  if let Some(selected) = selection {
    enter_account(account_cli, accounts[selected].clone());
    pick_account(account_cli, accounts)?;
  }
  Ok(())
}

fn select_account_command(account_cli: &App<'_>, manager: &AccountManager, matches: &ArgMatches) {
  if let Some(matches) = matches.subcommand_matches("account") {
    let alias = matches.value_of("alias").unwrap();
    if let Some(account) = manager.get_account_by_alias(alias) {
      enter_account(account_cli, account);
    } else {
      println!("Account not found");
    }
  }
}

fn new_account_command(
  account_cli: &App<'_>,
  manager: &AccountManager,
  matches: &ArgMatches,
) -> Result<()> {
  if let Some(matches) = matches.subcommand_matches("new") {
    let nodes: Vec<&str> = matches
      .values_of("node")
      .expect("at least a node must be provided")
      .collect();
    let mut builder = manager.create_account(ClientOptionsBuilder::nodes(&nodes)?.build().unwrap());
    if let Some(alias) = matches.value_of("alias") {
      builder = builder.alias(alias);
    }
    if let Some(mnemonic) = matches.value_of("mnemonic") {
      builder = builder.mnemonic(mnemonic);
    }
    let account = builder.initialise()?;
    println!("Created account `{}`", account.alias());
    enter_account(account_cli, account);
  }
  Ok(())
}

fn delete_account_command(manager: &AccountManager, matches: &ArgMatches) -> Result<()> {
  if let Some(matches) = matches.subcommand_matches("delete") {
    let account_id = matches.value_of("id").unwrap().to_string();
    manager.remove_account(account_id.into())?;
    println!("Account removed");
  }
  Ok(())
}

fn sync_accounts_command(manager: &AccountManager, matches: &ArgMatches) -> Result<()> {
  if let Some(_) = matches.subcommand_matches("sync") {
    let synced = block_on(async move { manager.sync_accounts().await })?;
    println!("Synchronized {} accounts", synced.len());
  }
  Ok(())
}

fn backup_command(manager: &AccountManager, matches: &ArgMatches) -> Result<()> {
  if let Some(matches) = matches.subcommand_matches("backup") {
    let destination = matches.value_of("path").unwrap();
    let full_path = manager.backup(destination)?;
    println!("Backup stored at {:?}", full_path);
  }
  Ok(())
}

fn import_command(manager: &AccountManager, matches: &ArgMatches) -> Result<()> {
  if let Some(matches) = matches.subcommand_matches("backup") {
    let source = matches.value_of("path").unwrap();
    manager.import_accounts(source)?;
    println!("Backup successfully imported");
  }
  Ok(())
}

fn main() -> Result<()> {
  let runtime = Runtime::new().expect("Failed to create async runtime");
  RUNTIME
    .set(Mutex::new(runtime))
    .expect("Failed to store async runtime");

  let mut manager = AccountManager::new()?;
  manager.set_stronghold_password("password")?;

  let yaml = load_yaml!("account-cli.yml");
  let account_cli = App::from(yaml)
    .help_template(ACCOUNT_CLI_TEMPLATE)
    .setting(AppSettings::DisableVersion)
    .setting(AppSettings::NoBinaryName);

  if std::env::args().len() == 1 {
    let accounts = manager.get_accounts()?;
    match accounts.len() {
      0 => {}
      1 => enter_account(&account_cli, accounts.first().unwrap().clone()),
      _ => pick_account(&account_cli, accounts)?,
    }
  }

  let yaml = load_yaml!("cli.yml");
  let matches = App::from(yaml)
    .help_template(CLI_TEMPLATE)
    .setting(AppSettings::ColoredHelp)
    .get_matches();

  select_account_command(&account_cli, &manager, &matches);
  new_account_command(&account_cli, &manager, &matches)?;
  delete_account_command(&manager, &matches)?;
  sync_accounts_command(&manager, &matches)?;
  backup_command(&manager, &matches)?;
  import_command(&manager, &matches)?;

  Ok(())
}
