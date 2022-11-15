// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import org.iota.Wallet;
import org.iota.types.*;
import org.iota.types.account_methods.SyncAccount;
import org.iota.types.exceptions.WalletException;
import org.iota.types.ids.account.AccountAlias;
import org.iota.types.secret.StrongholdSecretManager;

public class RecoverAccounts {
    private static final String DEFAULT_DEVELOPMENT_MNEMONIC = "hidden enroll proud copper decide negative orient asset speed work dolphin atom unhappy game cannon scheme glow kid ring core name still twist actor";

    public static void main(String[] args) throws WalletException {
        // This example assumes that a wallet has already been created using the ´CreateWallet.java´ example.
        // If you have not run the ´CreateAccount.java´ example yet, run it first to ensure that the wallet can be loaded correctly.
        Wallet wallet = new Wallet(new WalletConfig()
                .withClientOptions(new ClientConfig().withNodes(Env.NODE))
                .withSecretManager(
                        new StrongholdSecretManager(Env.STRONGHOLD_PASSWORD, null, Env.STRONGHOLD_SNAPSHOT_PATH))
                .withCoinType(CoinType.Shimmer)
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
    }
}