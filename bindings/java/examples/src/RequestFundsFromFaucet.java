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
        // This example assumes that a wallet has already been created using the ´CreateWallet.java´ example.
        // If you have not run the ´CreateAccount.java´ example yet, run it first to ensure that the wallet can be loaded correctly.
        Wallet wallet = new Wallet(new WalletConfig()
                .withClientOptions(new ClientConfig().withNodes(Env.NODE))
                .withSecretManager(
                        new StrongholdSecretManager(Env.STRONGHOLD_PASSWORD, null, Env.STRONGHOLD_SNAPSHOT_PATH))
                .withCoinType(CoinType.Shimmer)
        );

        // Get account and sync it with the registered node to ensure that its balances are up-to-date.
        AccountHandle a = wallet.getAccount(new AccountAlias(Env.ACCOUNT_NAME));
        AccountBalance balance = a.syncAccount(new SyncAccount().withOptions(new SyncOptions()));

        // Print the account balance before asking the faucet for funds.
        System.out.println("available account balance before faucet request: " + balance.getBaseCoin().getAvailable());

        // Get an address to fund.
        String address = a.getAddresses()[0].getAddress();

        // Request funds from the faucet.
        a.requestFundsFromFaucet(new org.iota.types.account_methods.RequestFundsFromFaucet("https://faucet.testnet.shimmer.network/api/enqueue", address));

        // Print the account balance after asking the faucet for funds.
        System.out.println("available account balance after faucet request: " + a.syncAccount(new SyncAccount()).getBaseCoin().getAvailable());
    }

}