// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_wallet::{account_manager::AccountManager, Error, StrongholdError};

const PBKDF_SALT: &str = "wallet.rs";
const PBKDF_ITER: u32 = 100;

#[cfg(feature = "stronghold")]
#[tokio::test]
async fn stronghold_snapshot_v2_v3_migration() -> Result<(), Error> {
    iota_stronghold::engine::snapshot::try_set_encrypt_work_factor(0).unwrap();

    let storage_path = "tests/fixtures/stronghold_snapshot_v2_v3_migration";
    // ./tests/wallet/fixtures/v2.stronghold
    let manager = AccountManager::builder()
        .with_storage(storage_path, None)?
        .with_skip_polling()
        .finish()
        .await?;

    assert!(matches!(
        manager.set_stronghold_password("current_password").await,
        Err(Error::StrongholdError(StrongholdError::UnsupportedSnapshotVersion { found, expected })) if found == 2 && expected == 3
    ));

    std::fs::copy(
        "tests/fixtures/stronghold_snapshot_v2_v3_migration/wallet.stronghold",
        "tests/fixtures/stronghold_snapshot_v2_v3_migration/copy.stronghold",
    )?;

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

    // let addresses = GetAddressesBuilder::new(&stronghold_secret_manager)
    //     .with_bech32_hrp(SHIMMER_TESTNET_BECH32_HRP)
    //     .with_coin_type(SHIMMER_COIN_TYPE)
    //     .with_account_index(0)
    //     .with_range(0..10)
    //     .finish()
    //     .await
    //     .unwrap();

    // // mnemonic: winter spend artefact viable cigar pink easy charge ranch license coyote cage brass mushroom repair
    // // game attack peanut glad rather cart obey famous chat
    // assert_eq!(
    //     addresses,
    //     [
    //         "rms1qrzyp87nqvcdctwrc7yzxjnwwetagffslhuknmey8t4fdf6552dnjxuaj3u",
    //         "rms1qpsvxm4q9p3xe4tkqjr04a64j0gxvhe0prt06vxwp0spkxfc8nr5gs28u0l",
    //         "rms1qqrt84z3dlhfy9wxa9whpn3xz9ugtspy80xwpu84p0cdszjc9vwr6d50m6k",
    //         "rms1qr57qle3rtj2kh5dtq8f00ys79cwa9dc3hq0hacd63aw0ngrx620vcctcza",
    //         "rms1qqkyhtt2lrqqpufcvydf6s7h8netyw0utuf0h458nafz26298wrespmrnyj",
    //         "rms1qz683r2zpl0qz355c3xlsrskke3563y9tn0s8u498zaxssr8ves0xq5p6c0",
    //         "rms1qrj4hszlpj6dnh3tpam5lwp0whgquj995ujsjvw0rxa5rt0sacrxxh4j9t7",
    //         "rms1qra52h296s4ty3x5np748xtruw52we63ardlp96v25yl9gzml7f7z8cvp9k",
    //         "rms1qqch88nnarx0czrdjee6v74ym08ruccr5w3wwxpk7nwjh3ll0dynxlnjtrw",
    //         "rms1qqrsl203x9wq29a2amcdszsps2lz7q20mqkh8t8vch0rz86pss9fwa8pjgx",
    //     ]
    // );

    // let restore_manager = Wallet::builder()
    //     .with_storage_path("test-storage/stronghold_snapshot_v2_v3_migration")
    //     .with_secret_manager(stronghold_secret_manager)
    //     .with_client_options(ClientOptions::new().with_node(NODE_LOCAL).unwrap())
    //     // Build with a different coin type, to check if it gets replaced by the one from the backup
    //     .with_coin_type(IOTA_COIN_TYPE)
    //     .finish()
    //     .await
    //     .unwrap();

    // // restore with ignore_if_coin_type_mismatch: Some(true) to not overwrite the coin type
    // let error = restore_manager
    //     .restore_backup(
    //         PathBuf::from("./tests/wallet/fixtures/v3.stronghold"),
    //         "wrong_password".to_string(),
    //         Some(false),
    //         None,
    //     )
    //     .await;

    // match error {
    //     Err(WalletError::Client(err)) => {
    //         assert!(matches!(
    //             *err,
    //             ClientError::Stronghold(StrongholdError::InvalidPassword)
    //         ));
    //     }
    //     _ => panic!("unexpected error"),
    // }

    std::fs::remove_file("tests/fixtures/stronghold_snapshot_v2_v3_migration/wallet.stronghold")?;
    std::fs::rename(
        "tests/fixtures/stronghold_snapshot_v2_v3_migration/copy.stronghold",
        "tests/fixtures/stronghold_snapshot_v2_v3_migration/wallet.stronghold",
    )?;
    std::fs::remove_dir_all("tests/fixtures/stronghold_snapshot_v2_v3_migration/db")?;

    Ok(())
}

// #[cfg(feature = "stronghold")]
// #[tokio::test]
// async fn stronghold_snapshot_v2_v3_migration_same_path() {
//     std::fs::copy(
//         "./tests/wallet/fixtures/v2.stronghold",
//         "./tests/wallet/fixtures/v2-copy.stronghold",
//     )
//     .unwrap();

//     let error = StrongholdSecretManager::builder()
//         .password("current_password")
//         .build("./tests/wallet/fixtures/v2-copy.stronghold");

//     assert!(matches!(
//         error,
//         Err(StrongholdError::UnsupportedSnapshotVersion { found, expected }) if found == 2 && expected == 3
//     ));

//     StrongholdAdapter::migrate_snapshot_v2_to_v3(
//         "./tests/wallet/fixtures/v2-copy.stronghold",
//         "current_password",
//         PBKDF_SALT,
//         PBKDF_ITER,
//         Some("./tests/wallet/fixtures/v2-copy.stronghold"),
//         Some("new_password"),
//     )
//     .unwrap();

//     StrongholdSecretManager::builder()
//         .password("new_password")
//         .build("./tests/wallet/fixtures/v2-copy.stronghold")
//         .unwrap();

//     std::fs::remove_file("./tests/wallet/fixtures/v2-copy.stronghold").unwrap();
// }

// #[cfg(feature = "stronghold")]
// #[tokio::test]
// async fn stronghold_snapshot_v2_v3_migration_with_backup() -> Result<(), Error> {
//     iota_stronghold::engine::snapshot::try_set_encrypt_work_factor(0).unwrap();

//     let storage_path = "tests/fixtures/stronghold_snapshot_v2_v3_migration_with_backup";
//     let manager = AccountManager::builder()
//         .with_storage(storage_path, None)?
//         .with_skip_polling()
//         .finish()
//         .await?;

//     assert!(matches!(
//         manager.set_stronghold_password("password").await,
//         Err(Error::StrongholdError(StrongholdError::UnsupportedSnapshotVersion { found, expected })) if found == 2 &&
// expected == 3     ));

//     std::fs::copy(
//         "tests/fixtures/stronghold_snapshot_v2_v3_migration_with_backup/wallet.stronghold",
//         "tests/fixtures/stronghold_snapshot_v2_v3_migration_with_backup/copy.stronghold",
//     )?;

//     AccountManager::migrate_stronghold_snapshot_v2_to_v3(
//         "tests/fixtures/stronghold_snapshot_v2_v3_migration_with_backup/wallet.stronghold",
//         "password",
//         PBKDF_SALT,
//         PBKDF_ITER,
//         None,
//         Some("new_password"),
//     )
//     .unwrap();

//     manager.set_stronghold_password("new_password").await?;

//     println!(
//         "{}",
//         iota_wallet::stronghold::get_record(
//             &std::path::PathBuf::from(
//                 "tests/fixtures/stronghold_snapshot_v2_v3_migration_with_backup/wallet.stronghold"
//             ),
//             "wallet-account://bbac12bf61b3eead23a9744e5964c15e49a888e1f0c64164cbfe86512257769b"
//         )
//         .await
//         .unwrap()
//     );

//     //     let coin_type_bytes = stronghold_secret_manager
//     //         .get("coin_type".as_bytes())
//     //         .await
//     //         .unwrap()
//     //         .expect("missing data");
//     //     let coin_type = u32::from_le_bytes(coin_type_bytes.try_into().expect("invalid coin_type"));
//     //     assert_eq!(coin_type, SHIMMER_COIN_TYPE);

//     //     let addresses = GetAddressesBuilder::new(&SecretManager::Stronghold(stronghold_secret_manager))
//     //         .with_bech32_hrp(SHIMMER_TESTNET_BECH32_HRP)
//     //         .with_coin_type(SHIMMER_COIN_TYPE)
//     //         .with_account_index(0)
//     //         .with_range(0..10)
//     //         .finish()
//     //         .await
//     //         .unwrap();

//     //     // mnemonic: brisk egg allow van merge process chest type dove bomb proud purity monitor snap load verb
// utility     //     // hungry cube coast fetch pioneer gadget credit
//     //     assert_eq!(
//     //         addresses,
//     //         [
//     //             "rms1qza3rek2ffhxtfjpaswfsc9hkekj7j5lrmzkp5fmr2wzlnz56hhvskcs4mz",
//     //             "rms1qqjpevw6d7spdzsmfrrzdna64fpdqh89jme8q4g47ek4l0kz3m5eqersz6g",
//     //             "rms1qqgc7rpa4u0uf4e085yap8ksz7jsrdqhy6saqszyt8etxleyph02spcdtsf",
//     //             "rms1qqv0pspup06eszwf9ne7xccxkx2eks6x5h8528cgxmnu382qnay7u8mkdfh",
//     //             "rms1qpqts58s8z6a0t3rs7z2q7qzr38h847rj3urc7esyqa0sdescg2sx553dct",
//     //             "rms1qrquh2afd0sx0sg26hamuksdm5sntzs0c903aptrv0lsfhehc0etg58j9wq",
//     //             "rms1qzzkwr6edw0pr25jzey7zmh5hkka4q4cqvvk5yhlgcg2ga7k8hzk2t0va50",
//     //             "rms1qp9mt8elk7x32npvvdtxnmdtt5n4wxe28zrwhc9hnyrr6jpsgp7dx4zp2nn",
//     //             "rms1qpt9gpycwmqy5ywrup8tmgpvrvxspqz7c9u9erk9qrwq72rk95y567p7j5z",
//     //             "rms1qqee8vjh3pqehpm5p53s45y4e7f5kusxnadt35hqyp5vvkrf8e3z2rrd3t9"
//     //         ]
//     //     );

//     std::fs::remove_file("tests/fixtures/stronghold_snapshot_v2_v3_migration_with_backup/wallet.stronghold")?;
//     std::fs::rename(
//         "tests/fixtures/stronghold_snapshot_v2_v3_migration_with_backup/copy.stronghold",
//         "tests/fixtures/stronghold_snapshot_v2_v3_migration_with_backup/wallet.stronghold",
//     )?;
//     std::fs::remove_dir_all("tests/fixtures/stronghold_snapshot_v2_v3_migration_with_backup/db")?;

//     Ok(())
// }
