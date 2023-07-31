// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_wallet::{account_manager::AccountManager, Error, StrongholdError};

const PBKDF_SALT: &str = "wallet.rs";
const PBKDF_ITER: u32 = 100;

#[cfg(feature = "stronghold")]
#[tokio::test]
async fn stronghold_snapshot_v2_v3_migration() -> Result<(), Error> {
    iota_stronghold::engine::snapshot::try_set_encrypt_work_factor(0).unwrap();

    // Remove potential files from previous run
    std::fs::remove_file("tests/fixtures/stronghold_snapshot_v2_v3_migration/wallet.stronghold").ok();
    std::fs::remove_dir_all("tests/fixtures/stronghold_snapshot_v2_v3_migration/db").ok();

    let storage_path = "tests/fixtures/stronghold_snapshot_v2_v3_migration";
    let manager = AccountManager::builder()
        .with_storage(storage_path, None)?
        .with_skip_polling()
        .finish()
        .await?;

    std::fs::copy(
        "tests/fixtures/stronghold_snapshot_v2_v3_migration/backup.stronghold",
        "tests/fixtures/stronghold_snapshot_v2_v3_migration/wallet.stronghold",
    )?;

    assert!(matches!(
        manager.set_stronghold_password("current_password").await,
        Err(Error::StrongholdError(StrongholdError::UnsupportedSnapshotVersion { found, expected })) if found == 2 && expected == 3
    ));

    AccountManager::migrate_stronghold_snapshot_v2_to_v3(
        "tests/fixtures/stronghold_snapshot_v2_v3_migration/wallet.stronghold",
        "current_password",
        PBKDF_SALT,
        PBKDF_ITER,
        None,
        Some("new_password"),
    )
    .unwrap();

    manager.set_stronghold_password("new_password").await?;

    let account = manager
        .create_account(iota_wallet::client::ClientOptionsBuilder::new().build().unwrap())?
        .initialise()
        .await?;

    // mnemonic: winter spend artefact viable cigar pink easy charge ranch license coyote cage brass mushroom repair
    // game attack peanut glad rather cart obey famous chat
    assert_eq!(
        account.addresses().await[0].address().to_bech32(),
        "atoi1qr2asvpxcvchjmt4rkkcmzscwtg8kqmkphgug9t0yazc3pvylclpy8jjrg0"
    );

    std::fs::remove_file("tests/fixtures/stronghold_snapshot_v2_v3_migration/wallet.stronghold")?;
    std::fs::remove_dir_all("tests/fixtures/stronghold_snapshot_v2_v3_migration/db")?;

    Ok(())
}

#[cfg(feature = "stronghold")]
#[tokio::test]
async fn stronghold_snapshot_v2_v3_migration_same_path() {
    std::fs::copy(
        "tests/fixtures/v2_with_backup.stronghold",
        "tests/fixtures/v2-copy.stronghold",
    )
    .unwrap();

    AccountManager::migrate_stronghold_snapshot_v2_to_v3(
        "tests/fixtures/v2-copy.stronghold",
        "current_password",
        PBKDF_SALT,
        PBKDF_ITER,
        Some("tests/fixtures/v2-copy.stronghold"),
        Some("new_password"),
    )
    .unwrap();

    std::fs::remove_file("tests/fixtures/v2-copy.stronghold").unwrap();
}

#[cfg(feature = "stronghold")]
#[tokio::test]
async fn stronghold_snapshot_v2_v3_migration_with_backup() -> Result<(), Error> {
    iota_stronghold::engine::snapshot::try_set_encrypt_work_factor(0).unwrap();

    let storage_path = "tests/fixtures/stronghold_snapshot_v2_v3_migration_with_backup";
    let manager = AccountManager::builder()
        .with_storage(storage_path, None)?
        .with_skip_polling()
        .finish()
        .await?;

    std::fs::copy(
        "tests/fixtures/stronghold_snapshot_v2_v3_migration_with_backup/backup.stronghold",
        "tests/fixtures/stronghold_snapshot_v2_v3_migration_with_backup/wallet.stronghold",
    )?;

    assert!(matches!(
    manager.set_stronghold_password("password").await,
    Err(Error::StrongholdError(StrongholdError::UnsupportedSnapshotVersion { found, expected })) if found == 2 &&
    expected == 3     ));

    AccountManager::migrate_stronghold_snapshot_v2_to_v3(
        "tests/fixtures/stronghold_snapshot_v2_v3_migration_with_backup/wallet.stronghold",
        "password",
        PBKDF_SALT,
        PBKDF_ITER,
        None,
        Some("new_password"),
    )
    .unwrap();

    std::fs::remove_dir_all("tests/fixtures/stronghold_snapshot_v2_v3_migration_with_backup/db")?;
    // Rename to wallet2, so it's not conflicting when restoring
    std::fs::rename(
        "tests/fixtures/stronghold_snapshot_v2_v3_migration_with_backup/wallet.stronghold",
        "tests/fixtures/stronghold_snapshot_v2_v3_migration_with_backup/wallet2.stronghold",
    )?;

    manager
        .import_accounts(
            "tests/fixtures/stronghold_snapshot_v2_v3_migration_with_backup/wallet2.stronghold",
            "new_password".to_string(),
        )
        .await?;

    let accounts = manager.get_accounts().await?;
    assert_eq!(accounts.len(), 1);

    std::fs::remove_file("tests/fixtures/stronghold_snapshot_v2_v3_migration_with_backup/wallet.stronghold")?;
    std::fs::remove_file("tests/fixtures/stronghold_snapshot_v2_v3_migration_with_backup/wallet2.stronghold")?;

    Ok(())
}
