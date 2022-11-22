// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import org.iota.Wallet;
import org.iota.types.*;
import org.iota.types.account_methods.SyncAccount;
import org.iota.types.exceptions.InitializeWalletException;
import org.iota.types.exceptions.WalletException;
import org.iota.types.ids.AliasId;
import org.iota.types.ids.account.AccountAlias;
import org.iota.types.secret.StrongholdSecretManager;

public class DestroyAliasOutput {
    public static void main(String[] args) throws WalletException, InterruptedException, InitializeWalletException {
        // This example assumes that a wallet has already been created using the ´CreateWallet.java´ example.
        // If you have not run the ´CreateAccount.java´ example yet, run it first to ensure that the wallet can be loaded correctly.
        Wallet wallet = new Wallet(new WalletConfig()
                .withClientOptions(new ClientConfig().withNodes("https://api.testnet.shimmer.network"))
                .withSecretManager(new StrongholdSecretManager("PASSWORD_FOR_ENCRYPTION", null, "example-wallet"))
                .withCoinType(CoinType.Shimmer)
        );

        // Get account and sync it with the registered node to ensure that its balances are up-to-date.
        AccountHandle a = wallet.getAccount(new AccountAlias("Alice"));
        a.syncAccount(new SyncAccount().withOptions(new SyncOptions()));

        // TODO: replace with your own values.
        AliasId aliasId = new AliasId("0xc3dc87b11c94ae919aae950309194bd145d8bf3a80824a63c24336dced7cb22f");

        // Send transaction.
        Transaction t = a.destroyAlias(new org.iota.types.account_methods.DestroyAlias().withAliasId(
                aliasId
        ));

        // Print transaction.
        System.out.println(t);
    }

}

