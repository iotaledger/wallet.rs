// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import org.iota.Wallet;
import org.iota.types.*;
import org.iota.types.account_methods.SyncAccount;
import org.iota.types.exceptions.WalletException;
import org.iota.types.ids.TokenId;
import org.iota.types.ids.account.AccountAlias;
import org.iota.types.secret.StrongholdSecretManager;

public class MintNativeToken {
    public static void main(String[] args) throws WalletException, InterruptedException {
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

        // Fund the account for this example.
        ExampleUtils.fundAccount(a);

        // TODO: replace with your own values.
        NativeTokenOptions options = new NativeTokenOptions();
        options.withCirculatingSupply("0x20"); // number 23 hex encoded
        options.withMaximumSupply("0x45"); // number 69 hex encoded

        // Send transaction.
        MintTokenTransaction t = a.mintNativeToken(new org.iota.types.account_methods.MintNativeToken().withNativeTokenOptions(options));

        // Print transaction.
        System.out.println(t);
    }

}