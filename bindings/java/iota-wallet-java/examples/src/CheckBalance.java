// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import org.iota.Wallet;
import org.iota.types.*;
import org.iota.types.ClientConfig.ApiTimeout;
import org.iota.types.account_methods.SyncAccount;
import org.iota.types.exceptions.InitializeWalletException;
import org.iota.types.exceptions.WalletException;
import org.iota.types.ids.account.AccountAlias;
import org.iota.types.secret.StrongholdSecretManager;

public class CheckBalance {
    public static void main(String[] args) throws WalletException, InterruptedException, InitializeWalletException {
        // This example assumes that a wallet has already been created using the ´CreateAccount.java´ example.
        // If you haven't run the ´CreateAccount.java´ example yet, you must run it first to be able to load the wallet as shown below:
        Wallet wallet = new Wallet(new WalletConfig()
                .withClientOptions(new ClientConfig().withNodes(Env.NODE).withApiTimeout(new ApiTimeout().withSecs(60)))
                .withSecretManager(
                        new StrongholdSecretManager(Env.STRONGHOLD_PASSWORD, null, Env.STRONGHOLD_SNAPSHOT_PATH))
                .withCoinType(CoinType.Shimmer)
        );

        // Get account and sync it with the registered node to ensure that its balances are up-to-date.
        AccountHandle a = wallet.getAccount(new AccountAlias(Env.ACCOUNT_NAME));
        a.syncAccount(new SyncAccount().withOptions(new SyncOptions()));

        // Get the balance.
        AccountBalance balance = a.getBalance();

        // Print the balance.
        System.out.println(balance);
    }
}