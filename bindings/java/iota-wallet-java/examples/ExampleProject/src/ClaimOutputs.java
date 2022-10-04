// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import org.iota.Wallet;
import org.iota.types.*;
import org.iota.types.exceptions.WalletException;
import org.iota.types.ids.OutputId;
import org.iota.types.secret.StrongholdSecretManager;

public class ClaimOutputs {
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

        // Claim the given outputs
        Transaction t = a.claimOutputs(new org.iota.types.account_methods.ClaimOutputs().withOutputIdsToClaim(new OutputId[]{
                new OutputId("0xcbcfbbd9b36bd9a3d6830311ed428036d13fc37050bcd1cf82fb4e4b695570fd0100")
        }));

        // Print the transaction
        System.out.println(t);
    }

}

