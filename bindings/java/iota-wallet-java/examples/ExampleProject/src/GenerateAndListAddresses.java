// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import org.iota.Wallet;
import org.iota.types.*;
import org.iota.types.account_methods.GenerateAddresses;
import org.iota.types.exceptions.WalletException;
import org.iota.types.ids.account.AccountAlias;
import org.iota.types.secret.MnemonicSecretManager;
public class GenerateAndListAddresses {
    private static final String SHIMMER_TESTNET_NODE_URL = "https://api.testnet.shimmer.network";
    private static final int SHIMMER_COIN_TYPE = 4219;
    private static final String DEFAULT_DEVELOPMENT_MNEMONIC = "hidden enroll proud copper decide negative orient asset speed work dolphin atom unhappy game cannon scheme glow kid ring core name still twist actor";

    public static void main(String[] args) throws WalletException {
        // Build the wallet.
        Wallet wallet = new Wallet(new WalletConfig()
                .withClientOptions(new ClientConfig().withNodes(SHIMMER_TESTNET_NODE_URL))
                .withSecretManager(new MnemonicSecretManager(DEFAULT_DEVELOPMENT_MNEMONIC))
                .withCoinType(SHIMMER_COIN_TYPE)
        );

        // Set up an account for this example.
        AccountHandle account = wallet.createAccount("Hans");

        // Generate two addresses.
        AccountAddress[] addresses = account.generateAddresses(new GenerateAddresses().withAmount(2));

        // Print the generated addresses.
        for (AccountAddress address : addresses)
            System.out.println(address.getAddress());

        // Print all addresses.
        for (AccountAddress address : account.getPublicAddresses())
            System.out.println(address.getAddress());
    }
}
