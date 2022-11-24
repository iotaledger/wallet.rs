// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import org.iota.Wallet;
import org.iota.types.*;
import org.iota.types.account_methods.SyncAccount;
import org.iota.types.exceptions.InitializeWalletException;
import org.iota.types.exceptions.WalletException;
import org.iota.types.ids.TokenId;
import org.iota.types.ids.account.AccountAlias;
import org.iota.types.secret.StrongholdSecretManager;

public class BurnNativeToken {

    public static void main(String[] args) throws WalletException, InterruptedException, InitializeWalletException {
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
        TokenId tokenId = new TokenId("0x08429fe5864378ce70699fc2d22bb144cb86a3c4833d136e3b95c5dadfd6ba0cef0300000000");
        String burnAmount = "0x5";

        // Send transaction.
        Transaction t = a.burnNativeToken(new org.iota.types.account_methods.BurnNativeToken()
                .withTokenId(tokenId)
                .withBurnAmount(burnAmount)
        );

        // Print transaction.
        System.out.println(t);
    }
}