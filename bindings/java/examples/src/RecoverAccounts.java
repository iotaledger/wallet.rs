// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import org.iota.Wallet;
import org.iota.types.ClientConfig;
import org.iota.types.CoinType;
import org.iota.types.SyncOptions;
import org.iota.types.WalletConfig;
import org.iota.types.exceptions.InitializeWalletException;
import org.iota.types.exceptions.WalletException;
import org.iota.types.secret.StrongholdSecretManager;

public class RecoverAccounts {
    public static void main(String[] args) throws WalletException, InitializeWalletException {
        // This example assumes that a wallet has already been created using the ´SetupWallet.java´ example.
        // If you haven't run the ´SetupWallet.java´ example yet, you must run it first to be able to load the wallet as shown below:
        Wallet wallet = new Wallet(new WalletConfig()
                .withClientOptions(new ClientConfig().withNodes(Env.NODE))
                .withSecretManager(new StrongholdSecretManager(Env.STRONGHOLD_PASSWORD, null, Env.STRONGHOLD_VAULT_PATH))
                .withCoinType(CoinType.Shimmer)
                .withStoragePath(Env.STORAGE_PATH)
        );

        // Search for accounts with unspent outputs

        // The index of the first account to search for.
        int accountStartIndex = 0;
        // The number of accounts to search for, after the last account with unspent outputs.
        int accountGapLimit = 5;
        // The number of addresses to search for, after the last address with unspent outputs, in
        /// each account.
        int addressGapLimit = 10;
        // Optional parameter to specify the sync options. The `address_start_index` and `force_syncing` fields will be overwritten to skip existing addresses.
        SyncOptions syncOptions = null;

        wallet.recoverAccounts(accountStartIndex, accountGapLimit, addressGapLimit, syncOptions);

        // In case you are done and don't need the wallet instance anymore you can destroy the instance to clean up memory.
        // For this, check out the ´DestroyWallet.java´ example.
    }
}