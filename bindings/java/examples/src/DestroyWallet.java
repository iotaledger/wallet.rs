// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import org.iota.Wallet;
import org.iota.types.ClientConfig;
import org.iota.types.CoinType;
import org.iota.types.WalletConfig;
import org.iota.types.exceptions.InitializeWalletException;
import org.iota.types.exceptions.WalletException;
import org.iota.types.secret.StrongholdSecretManager;

public class DestroyWallet {

    public static void main(String[] args) throws WalletException, InterruptedException, InitializeWalletException {
        // This example assumes that a wallet has already been created using the ´SetupWallet.java´ example.
        // If you haven't run the ´SetupWallet.java´ example yet, you must run it first to be able to load the wallet as shown below:
        Wallet wallet = new Wallet(new WalletConfig()
                .withClientOptions(new ClientConfig().withNodes(Env.NODE))
                .withSecretManager(new StrongholdSecretManager(Env.STRONGHOLD_PASSWORD, null, Env.STRONGHOLD_VAULT_PATH))
                .withCoinType(CoinType.Shimmer)
                .withStoragePath(Env.STORAGE_PATH)
        );

        // Destroy wallet instance and clean up memory.
        wallet.destroy();

        try {
            // Calling this will throw an error as the instance is cleaned up
            wallet.generateMnemonic();
        } catch (NullPointerException e) {
            // Will print "Wallet not initialised"
            System.out.println(e.getMessage());
        }
    }
}
