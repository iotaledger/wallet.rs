// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import org.iota.Wallet;
import org.iota.types.*;
import org.iota.types.account_methods.MintNfts;
import org.iota.types.account_methods.SyncAccount;
import org.iota.types.exceptions.InitializeWalletException;
import org.iota.types.exceptions.WalletException;
import org.iota.types.ids.account.AccountAlias;
import org.iota.types.secret.StrongholdSecretManager;

public class MintNft {
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

        // Fund the account for this example.
        ExampleUtils.fundAccount(a);

        // TODO: replace with your own values.
        NftOptions options = new NftOptions();
        options.withMetadata("0x5368696d6d65722e20546f6b656e697a652045766572797468696e672e2048656c6c6f2066726f6d20746865204a6176612062696e64696e672e");

        // Send transaction.
        Transaction t = a.mintNfts(new MintNfts().withNftsOptions(new NftOptions[]{options}));

        // Print the transaction.
        System.out.println(t);
    }

}