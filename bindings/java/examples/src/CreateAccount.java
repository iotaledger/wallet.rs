// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import org.iota.Wallet;
import org.iota.types.AccountHandle;
import org.iota.types.ClientConfig;
import org.iota.types.CoinType;
import org.iota.types.WalletConfig;
import org.iota.types.exceptions.InitializeWalletException;
import org.iota.types.exceptions.WalletException;
import org.iota.types.secret.StrongholdSecretManager;

public class CreateAccount {
    public static void main(String[] args) throws WalletException, InitializeWalletException {
        // This example assumes that a wallet has already been created using the ´SetupWallet.java´ example.
        // If you haven't run the ´SetupWallet.java´ example yet, you must run it first to be able to load the wallet as shown below:
        Wallet wallet = new Wallet(new WalletConfig()
                .withClientOptions(new ClientConfig().withNodes(Env.NODE))
                .withSecretManager(new StrongholdSecretManager(Env.STRONGHOLD_PASSWORD, null, Env.STRONGHOLD_VAULT_PATH))
                .withCoinType(CoinType.Shimmer)
                .withStoragePath(Env.STORAGE_PATH)
        );

        // Create an account.
        AccountHandle a = wallet.createAccount(Env.ACCOUNT_NAME);

        // Print the account.
        System.out.println(a);

        // In case you are done and don't need the wallet instance anymore you can destroy the instance to clean up memory.
        // For this, check out the ´DestroyWallet.java´ example.
    }
}