// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import org.iota.Wallet;
import org.iota.types.Account;
import org.iota.types.AccountHandle;
import org.iota.types.ClientConfig;
import org.iota.types.WalletConfig;
import org.iota.types.exceptions.WalletException;
import org.iota.types.ids.account.AccountAlias;
import org.iota.types.ids.account.AccountIndex;
import org.iota.types.secret.MnemonicSecretManager;
import org.iota.types.secret.StrongholdSecretManager;

public class GetAccountByIndex {
    private static final String SHIMMER_TESTNET_NODE_URL = "https://api.testnet.shimmer.network";
    private static final int SHIMMER_COIN_TYPE = 4219;
    private static final String DEFAULT_DEVELOPMENT_MNEMONIC = "hidden enroll proud copper decide negative orient asset speed work dolphin atom unhappy game cannon scheme glow kid ring core name still twist actor";

    public static void main(String[] args) throws WalletException {
        // Build the wallet.
        Wallet wallet = new Wallet(new WalletConfig()
                .withClientOptions(new ClientConfig().withNodes(SHIMMER_TESTNET_NODE_URL))
                .withSecretManager(new StrongholdSecretManager("PASSWORD_FOR_ENCRYPTION", 5, "example-wallet"))
                .withCoinType(SHIMMER_COIN_TYPE)
        );
        wallet.storeMnemonic(DEFAULT_DEVELOPMENT_MNEMONIC);

        // Set up an account for this example.
        wallet.createAccount("Hans");

        // Get the account by index.
        AccountHandle a = wallet.getAccount(new AccountIndex(0));

        // Print the account.
        System.out.println(a);
    }
}
