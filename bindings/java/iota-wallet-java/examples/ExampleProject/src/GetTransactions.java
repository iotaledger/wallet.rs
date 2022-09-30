// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import org.iota.Wallet;
import org.iota.types.*;
import org.iota.types.account_methods.GetTransaction;
import org.iota.types.exceptions.WalletException;
import org.iota.types.ids.account.AccountAlias;
import org.iota.types.secret.StrongholdSecretManager;

public class GetTransactions {
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

        // Set up a transaction for this example.
        AccountAddress address = wallet.getAccount(new AccountAlias("Alice")).getPublicAddresses()[0];
        a.sendAmount(new org.iota.types.account_methods.SendAmount().withAddressesWithAmount(
                new AddressWithAmount[]{new AddressWithAmount().withAddress(address.getAddress()).withAmount("1000000")}
        ));

        // List transactions
        Transaction[] transactions = a.getTransactions();

        // Print transactions
        for (Transaction tx : transactions)
            System.out.println(tx.toString());
    }

}

