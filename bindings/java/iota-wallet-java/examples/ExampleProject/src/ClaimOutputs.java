// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import org.iota.Wallet;
import org.iota.types.*;
import org.iota.types.account_methods.SyncAccount;
import org.iota.types.exceptions.WalletException;
import org.iota.types.ids.OutputId;
import org.iota.types.ids.account.AccountAlias;
import org.iota.types.secret.StrongholdSecretManager;

public class ClaimOutputs {
    public static void main(String[] args) throws WalletException, InterruptedException {
        // This example assumes that a wallet has already been created using the ´CreateAccount.java´ example.
        // If you haven't run the ´CreateAccount.java´ example yet, you must run it first to be able to load the wallet as shown below:
        Wallet wallet = new Wallet(new WalletConfig()
                .withClientOptions(new ClientConfig().withNodes("https://api.testnet.shimmer.network"))
                .withSecretManager(new StrongholdSecretManager("PASSWORD_FOR_ENCRYPTION", null, "example-wallet"))
                .withCoinType(CoinType.Shimmer)
        );

        // Get account and sync it with the registered node to ensure that its balances are up-to-date.
        AccountHandle a = wallet.getAccount(new AccountAlias("Alice"));
        a.syncAccount(new SyncAccount().withOptions(new SyncOptions()));

        // TODO: replace with your own values.
        OutputId outputId = new OutputId("0xcbcfbbd9b36bd9a3d6830311ed428036d13fc37050bcd1cf82fb4e4b695570fd0100");

        // Claim the given outputs
        Transaction t = a.claimOutputs(new org.iota.types.account_methods.ClaimOutputs().withOutputIdsToClaim(new OutputId[]{
                outputId
        }));

        // Print the transaction
        System.out.println(t);
    }
}