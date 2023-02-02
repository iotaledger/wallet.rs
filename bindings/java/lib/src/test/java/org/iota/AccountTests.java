// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0
package org.iota;

import org.iota.types.AccountAddress;
import org.iota.types.AccountHandle;
import org.iota.types.account_methods.SetAlias;
import org.iota.types.exceptions.WalletException;
import org.iota.types.ids.account.AccountIndex;
import org.junit.jupiter.api.Test;

public class AccountTests extends TestSettings {

    @Test
    public void testListAddresses() throws WalletException {
        AccountHandle a = wallet.createAccount("Alice");
        AccountAddress[] addresses = a.getAddresses();
        for(AccountAddress address : addresses) {
            System.out.println(address.getAddress());
        }
    }

    @Test
    public void testSetAlias() throws WalletException {
        AccountHandle a = wallet.createAccount("Alice");
        a.setAlias(new SetAlias().withAlias("Bob"));
        System.out.println(wallet.getAccount(new AccountIndex(0)).getAlias());
    }

}
