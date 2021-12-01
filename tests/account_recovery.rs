use wallet_core::{account_manager::AccountManager, client::options::ClientOptionsBuilder, Result};

// can't be run together with all other tests because there can be only one mnemonic at a time
#[ignore]
#[tokio::test]
async fn account_recovery_empty() -> Result<()> {
    let client_options = ClientOptionsBuilder::new()
        .with_node("https://api.lb-0.h.chrysalis-devnet.iota.cafe")?
        .with_node("https://api.thin-hornet-0.h.chrysalis-devnet.iota.cafe")?
        .with_node("https://api.thin-hornet-1.h.chrysalis-devnet.iota.cafe")?
        .with_node_sync_disabled()
        .finish()?;

    let manager = AccountManager::builder()
        .with_client_options(client_options)
        .finish()
        .await?;

    // mnemonic without balance
    let mnemonic = "inhale gorilla deny three celery song category owner lottery rent author wealth penalty crawl hobby obtain glad warm early rain clutch slab august bleak".to_string();
    manager.store_mnemonic(Some(mnemonic)).await?;

    let accounts = manager.recover_accounts(2, 2).await?;
    // accounts should be empty if no account was created before and no account was found with balance
    assert_eq!(0, accounts.len());
    Ok(())
}

// can't be run together with all other tests because there can be only one mnemonic at a time
#[ignore]
#[tokio::test]
async fn account_recovery_existing_accounts() -> Result<()> {
    let client_options = ClientOptionsBuilder::new()
        .with_node("https://api.lb-0.h.chrysalis-devnet.iota.cafe")?
        .with_node("https://api.thin-hornet-0.h.chrysalis-devnet.iota.cafe")?
        .with_node("https://api.thin-hornet-1.h.chrysalis-devnet.iota.cafe")?
        .with_node_sync_disabled()
        .finish()?;

    let manager = AccountManager::builder()
        .with_client_options(client_options)
        .finish()
        .await?;

    // mnemonic without balance
    let mnemonic = "inhale gorilla deny three celery song category owner lottery rent author wealth penalty crawl hobby obtain glad warm early rain clutch slab august bleak".to_string();
    manager.store_mnemonic(Some(mnemonic)).await?;

    // create two accounts
    manager.create_account().finish().await?;
    manager.create_account().finish().await?;

    let accounts = manager.recover_accounts(2, 2).await?;

    // accounts should still be ordered
    for (index, account) in accounts.iter().enumerate() {
        assert_eq!(&index, account.read().await.index());
    }
    // accounts should be 2 because we created 2 accounts before and no new account was found with balance
    assert_eq!(2, accounts.len());
    Ok(())
}

// can't be run together with all other tests because there can be only one mnemonic at a time
#[ignore]
#[tokio::test]
async fn account_recovery_with_balance() -> Result<()> {
    let client_options = ClientOptionsBuilder::new()
        .with_node("https://api.lb-0.h.chrysalis-devnet.iota.cafe")?
        .with_node("https://api.thin-hornet-0.h.chrysalis-devnet.iota.cafe")?
        .with_node("https://api.thin-hornet-1.h.chrysalis-devnet.iota.cafe")?
        .with_node_sync_disabled()
        .finish()?;

    let manager = AccountManager::builder()
        .with_client_options(client_options)
        .finish()
        .await?;

    // mnemonic with balance on account with index 2 and address key_index 2 on the public address
    // atoi1qqt9tygh7h7s3l66m242hee6zwp98x90trejt9zya4vcnf5u34yluws9af6
    let mnemonic = "merit blame slam front add unknown winner wait matrix carbon lion cram picnic mushroom turn stadium bright wheel open tragic liar will law time".to_string();
    manager.store_mnemonic(Some(mnemonic)).await?;

    // create one account
    manager.create_account().finish().await?;

    let accounts = manager.recover_accounts(3, 2).await?;

    // accounts should still be ordered
    for (index, account) in accounts.iter().enumerate() {
        assert_eq!(&index, account.read().await.index());
    }
    // accounts should be 3 because account with index 2 has balance
    assert_eq!(3, accounts.len());

    let account_with_balance = accounts[2].read().await;
    // should have 3 addresses, index 0, 1, 2, all above should be deleted again
    assert_eq!(3, account_with_balance.public_addresses().len());
    Ok(())
}

// can't be run together with all other tests because there can be only one mnemonic at a time
#[ignore]
#[tokio::test]
async fn account_recovery_with_balance_and_empty_addresses() -> Result<()> {
    let client_options = ClientOptionsBuilder::new()
        .with_node("https://api.lb-0.h.chrysalis-devnet.iota.cafe")?
        .with_node("https://api.thin-hornet-0.h.chrysalis-devnet.iota.cafe")?
        .with_node("https://api.thin-hornet-1.h.chrysalis-devnet.iota.cafe")?
        .with_node_sync_disabled()
        .finish()?;

    let manager = AccountManager::builder()
        .with_client_options(client_options)
        .finish()
        .await?;

    // mnemonic with balance on account with index 2 and address key_index 2 on the public address
    // atoi1qqt9tygh7h7s3l66m242hee6zwp98x90trejt9zya4vcnf5u34yluws9af6
    let mnemonic = "merit blame slam front add unknown winner wait matrix carbon lion cram picnic mushroom turn stadium bright wheel open tragic liar will law time".to_string();
    manager.store_mnemonic(Some(mnemonic)).await?;

    // create one account
    manager.create_account().finish().await?;
    manager.create_account().finish().await?;
    let account = manager.create_account().finish().await?;
    let _addresses = account.generate_addresses(5, None).await?;

    let accounts = manager.recover_accounts(3, 2).await?;

    // accounts should still be ordered
    for (index, account) in accounts.iter().enumerate() {
        assert_eq!(&index, account.read().await.index());
    }
    // accounts should be 3 because account with index 2 has balance
    assert_eq!(3, accounts.len());

    let account_with_balance = accounts[2].read().await;
    // should have 10 addresses, because we generated 10 before, even thought they don't all have funds
    assert_eq!(5, account_with_balance.public_addresses().len());
    Ok(())
}
