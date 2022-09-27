// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import org.iota.Wallet;
import org.iota.types.AccountHandle;
import org.iota.types.account_methods.SyncAccount;
import org.iota.types.exceptions.WalletException;

public class ExampleUtils {

    public static AccountHandle setUpAccountWithFunds(Wallet wallet, String alias) throws WalletException, InterruptedException {
        AccountHandle a = wallet.createAccount(alias);

        int available = a.syncAccount(new SyncAccount()).getBaseCoin().getAvailable();

        while (available < 1000000) {
            System.out.println("Please fund following address: " + a.getPublicAddresses()[0]);
            Thread.sleep(5000);
        }

        return a;
    }

}
