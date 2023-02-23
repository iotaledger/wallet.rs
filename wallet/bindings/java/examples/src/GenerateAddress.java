// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import org.iota.Wallet;
import org.iota.types.*;
import org.iota.types.account_methods.GenerateAddresses;
import org.iota.types.account_methods.SyncAccount;
import org.iota.types.exceptions.InitializeWalletException;
import org.iota.types.exceptions.WalletException;
import org.iota.types.ids.account.AccountAlias;
import org.iota.types.secret.StrongholdSecretManager;

public class GenerateAddress {
    public static void main(String[] args) throws WalletException, InitializeWalletException {
        // This example assumes that a wallet has already been created using the ´SetupWallet.java´ example.
        // If you haven't run the ´SetupWallet.java´ example yet, you must run it first to be able to load the wallet as shown below:
        Wallet wallet = new Wallet(new WalletConfig()
                .withClientOptions(new ClientConfig().withNodes(Env.NODE))
                .withSecretManager(new StrongholdSecretManager(Env.STRONGHOLD_PASSWORD, null, Env.STRONGHOLD_VAULT_PATH))
                .withCoinType(CoinType.Shimmer)
                .withStoragePath(Env.STORAGE_PATH)
        );

        // Get account and sync it with the registered node to ensure that its balances are up-to-date.
        AccountHandle a = wallet.getAccount(new AccountAlias(Env.ACCOUNT_NAME));
        a.syncAccount(new SyncAccount().withOptions(new SyncOptions()));

        // Generate two addresses.
        AccountAddress[] addresses = a.generateAddresses(new GenerateAddresses().withAmount(2));

        // Print the generated addresses.
        for (AccountAddress address : addresses)
            System.out.println(address.getAddress());

        // In case you are done and don't need the wallet instance anymore you can destroy the instance to clean up memory.
        // For this, check out the ´DestroyWallet.java´ example.
    }
}