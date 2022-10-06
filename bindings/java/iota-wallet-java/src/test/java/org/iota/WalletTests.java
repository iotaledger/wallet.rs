// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0
package org.iota;

import org.iota.types.AccountAddress;
import org.iota.types.AccountHandle;
import org.iota.types.account_methods.SetAlias;
import org.iota.types.exceptions.WalletException;
import org.iota.types.ids.account.AccountAlias;
import org.iota.types.ids.account.AccountIndex;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
public class WalletTests extends TestSettings {
    @Test
    public void testCreateAccount() throws WalletException {
        System.out.println(wallet.createAccount("Alice"));
    }

    @Test
    public void testCreateAccountsWithSameAlias() throws WalletException {
        System.out.println(wallet.createAccount("Alice"));
        try {
            System.out.println(wallet.createAccount("Alice"));
        } catch (WalletException expectedException) { ; }
    }

    @Test
    public void testGetAccountByAlias() throws WalletException {
        AccountHandle a = wallet.createAccount("Alice");
        AccountHandle b = wallet.getAccount(new AccountAlias("Alice"));
        assertEquals(a,b);
    }

    @Test
    public void testGetAccountByIndex() throws WalletException {
        AccountHandle a = wallet.createAccount("Alice");
        AccountHandle b = wallet.getAccount(new AccountIndex(0));
        assertEquals(a,b);
    }

    @Test
    public void testGetAccounts() throws WalletException {
        AccountHandle a = wallet.createAccount("Alice");
        AccountHandle b = wallet.createAccount("Bob");
        assertTrue(wallet.getAccounts().length == 2);
        for (AccountHandle x : wallet.getAccounts())
            System.out.println(x);
    }

}
