//! Wallet CLI example
//! Create a new account: `$ cargo run --example cli -- new --node http://localhost:14265`

use anyhow::Result;
use clap::{load_yaml, App, AppSettings, ArgMatches, ErrorKind as ClapErrorKind};
use console::Term;
use dialoguer::{theme::ColorfulTheme, Input, Select};
use futures::future::{BoxFuture, FutureExt};
use iota_wallet::{
  account::Account, account_manager::AccountManager, client::ClientOptionsBuilder,
  message::MessageType,
};

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

fn list_messages_command(account: &Account, matches: &ArgMatches) {
  if let Some(matches) = matches.subcommand_matches("messages") {
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
      println!("no messages found");
    } else {
      messages.iter().for_each(|m| println!("{:?}\n\n", m));
    }
  }
}

fn list_addresses_command(account: &Account, matches: &ArgMatches) {
  if let Some(_) = matches.subcommand_matches("addresses") {
    let mut addresses = account.list_addresses(false);
    addresses.extend(account.list_addresses(true));
    if addresses.is_empty() {
      println!("no addresses found");
    } else {
      addresses.iter().for_each(|m| println!("{:?}\n\n", m));
    }
  }
}

fn synchronize_command(account: &mut Account, matches: &ArgMatches) {
  if matches.subcommand_matches("sync").is_some() {
    let mut runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async move {
      account
        .sync()
        .execute()
        .await
        .expect("failed to synchronize account with the Tangle");
    });
  }
}

fn enter_account(account_cli: App<'_>, mut account: Account) {
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

          list_messages_command(&account, &matches);
          list_addresses_command(&account, &matches);
          synchronize_command(&mut account, &matches);
        }
        Err(e) => {
          let mut cli = account_cli.clone();
          match e.kind {
            ClapErrorKind::DisplayHelp => cli.print_help().unwrap(),
            ClapErrorKind::DisplayVersion => println!("{}", cli.render_version()),
            _ => {
              println!("{}", e.to_string());
            }
          };
        }
      }
    }
  }

  enter_account(account_cli, account);
}

fn pick_account(account_cli: App<'_>, accounts: Vec<Account>) -> BoxFuture<'_, Result<()>> {
  async move {
    let items: Vec<&String> = accounts.iter().map(|acc| acc.alias()).collect();
    let selection = Select::with_theme(&ColorfulTheme::default())
      .with_prompt("Select an account to manipulate")
      .items(&items)
      .default(0)
      .interact_on_opt(&Term::stderr())?;
    if let Some(selected) = selection {
      enter_account(account_cli.clone(), accounts[selected].clone());
      pick_account(account_cli, accounts).await?;
    }
    Ok(())
  }
  .boxed()
}

async fn new_account_command(account_cli: App<'_>, manager: &AccountManager, matches: &ArgMatches) {
  if let Some(matches) = matches.subcommand_matches("new") {
    let nodes: Vec<&str> = matches
      .values_of("node")
      .expect("at least a node must be provided")
      .collect();
    let mut builder = manager.create_account(
      ClientOptionsBuilder::nodes(&nodes)
        .expect("invalid node url")
        .build()
        .unwrap(),
    );
    if let Some(alias) = matches.value_of("alias") {
      builder = builder.alias(alias);
    }
    if let Some(mnemonic) = matches.value_of("mnemonic") {
      builder = builder.mnemonic(mnemonic);
    }
    let account = builder.initialise().expect("failed to create account");
    println!("created account `{}`", account.alias());
    enter_account(account_cli, account);
  }
}

#[tokio::main]
async fn main() -> Result<()> {
  let mut manager = AccountManager::new().unwrap();
  manager.set_stronghold_password("password")?;

  let yaml = load_yaml!("account-cli.yml");
  let account_cli = App::from(yaml)
    .help_template(ACCOUNT_CLI_TEMPLATE)
    .setting(AppSettings::NoBinaryName);

  if std::env::args().len() == 1 {
    let accounts = manager.get_accounts()?;
    match accounts.len() {
      0 => {}
      1 => enter_account(account_cli.clone(), accounts.first().unwrap().clone()),
      _ => pick_account(account_cli.clone(), accounts).await?,
    }
  }

  let yaml = load_yaml!("cli.yml");
  let matches = App::from(yaml).help_template(CLI_TEMPLATE).get_matches();

  new_account_command(account_cli, &manager, &matches).await;

  Ok(())
}
