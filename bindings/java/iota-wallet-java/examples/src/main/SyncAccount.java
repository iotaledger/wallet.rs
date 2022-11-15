// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import org.iota.Wallet;
import org.iota.types.*;
import org.iota.types.exceptions.WalletException;
import org.iota.types.ids.account.AccountAlias;
import org.iota.types.secret.StrongholdSecretManager;

public class SyncAccount {
    public static void main(String[] args) throws WalletException, InterruptedException {
        // This example assumes that a wallet has already been created using the ´CreateWallet.java´ example.
        // If you have not run the ´CreateAccount.java´ example yet, run it first to ensure that the wallet can be loaded correctly.
        Wallet wallet = new Wallet(new WalletConfig()
                .withClientOptions(new ClientConfig().withNodes(Env.NODE))
                .withSecretManager(
                        new StrongholdSecretManager(Env.STRONGHOLD_PASSWORD, null, Env.STRONGHOLD_SNAPSHOT_PATH))
                .withCoinType(CoinType.Shimmer)
        );

        // Get account.
        AccountHandle a = wallet.getAccount(new AccountAlias(Env.ACCOUNT_NAME));

        // Sync the account with the registered node to ensure that its balances are up-to-date.
        AccountBalance balance = a.syncAccount(new org.iota.types.account_methods.SyncAccount().withOptions(new SyncOptions()));

        // Print the balance.
        System.out.println(balance.getBaseCoin().getTotal());
    }
}