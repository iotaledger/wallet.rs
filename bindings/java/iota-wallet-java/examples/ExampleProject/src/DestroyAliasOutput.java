// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import org.iota.Wallet;
import org.iota.types.*;
import org.iota.types.account_methods.SyncAccount;
import org.iota.types.exceptions.WalletException;
import org.iota.types.ids.AliasId;
import org.iota.types.ids.FoundryId;
import org.iota.types.secret.StrongholdSecretManager;

public class DestroyAliasOutput {
    private static final String DEFAULT_DEVELOPMENT_MNEMONIC = "hidden enroll proud copper decide negative orient asset speed work dolphin atom unhappy game cannon scheme glow kid ring core name still twist actor";

    public static void main(String[] args) throws WalletException, InterruptedException {
        // Build the wallet.
        Wallet wallet = new Wallet(new WalletConfig()
                .withClientOptions(new ClientConfig().withNodes("https://api.testnet.shimmer.network"))
                .withSecretManager(new StrongholdSecretManager("PASSWORD_FOR_ENCRYPTION", null, "example-wallet"))
                .withCoinType(CoinType.Shimmer)
        );
        wallet.storeMnemonic(DEFAULT_DEVELOPMENT_MNEMONIC);

        // Set up an account for this example.
        AccountHandle a = ExampleUtils.setUpAccountWithFunds(wallet, "Alice");

        // Sync account
        AccountBalance b = a.syncAccount(new SyncAccount().withOptions(new SyncOptions()));

        // Create transaction
        Transaction t = a.destroyAlias(new org.iota.types.account_methods.DestroyAlias()
                .withAliasId(new AliasId("0x429fe5864378ce70699fc2d22bb144cb86a3c4833d136e3b95c5dadfd6ba0cef"))
        );

        // Print Native Token transaction
        System.out.println(t);
    }

}

