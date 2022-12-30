// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import org.iota.Wallet;
import org.iota.types.*;
import org.iota.types.account_methods.SyncAccount;
import org.iota.types.exceptions.InitializeWalletException;
import org.iota.types.exceptions.NoFundsReceivedFromFaucetException;
import org.iota.types.exceptions.WalletException;
import org.iota.types.ids.account.AccountAlias;
import org.iota.types.secret.StrongholdSecretManager;

public class RequestFundsFromFaucet {
    
    public static void main(String[] args) throws WalletException, InterruptedException, InitializeWalletException, NoFundsReceivedFromFaucetException {
        // This example assumes that a wallet has already been created using the ´SetupWallet.java´ example.
        // If you haven't run the ´SetupWallet.java´ example yet, you must run it first to be able to load the wallet as shown below:
        Wallet wallet = new Wallet(new WalletConfig()
                .withClientOptions(new ClientConfig().withNodes("https://api.testnet.shimmer.network"))
                .withSecretManager(new StrongholdSecretManager(Env.STRONGHOLD_PASSWORD, null, Env.STRONGHOLD_VAULT_PATH))
                .withCoinType(CoinType.Shimmer)
                .withStoragePath(Env.STORAGE_PATH)
        );

        // Get account and sync it with the registered node to ensure that its balances are up-to-date.
        AccountHandle a = wallet.getAccount(new AccountAlias(Env.ACCOUNT_NAME));
        AccountBalance balance = a.syncAccount(new SyncAccount().withOptions(new SyncOptions()));

        // Print the account balance before asking the faucet for funds.
        System.out.println("available account balance before faucet request: " + balance.getBaseCoin().getAvailable());

        // Get an address to fund.
        String address = a.getAddresses()[0].getAddress();

        // Syncs the account with the provided sync options and request funds from the faucet.
        a.requestFundsFromFaucet(new org.iota.types.account_methods.RequestFundsFromFaucet("https://faucet.testnet.shimmer.network/api/enqueue", address), 10000000, new SyncOptions());

        // Print the account balance after asking the faucet for funds.
        System.out.println("available account balance after faucet request: " + a.syncAccount(new SyncAccount()).getBaseCoin().getAvailable());

        // In case you are done and don't need the wallet instance anymore you can destroy the instance to clean up memory.
        // For this, check out the ´DestroyWallet.java´ example.
    }

}