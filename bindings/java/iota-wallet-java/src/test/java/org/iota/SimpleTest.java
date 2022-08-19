package org.iota;

import org.iota.types.Account;
import org.iota.types.WalletException;
import org.iota.types.ids.account.AccountAlias;
import org.iota.types.ids.account.AccountIndex;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class SimpleTest extends ApiTest {

    @Test
    public void testCreateAccount() throws WalletException {
        System.out.println(wallet.createAccount("Hans"));
    }

    @Test
    public void testGetAccountByAlias() throws WalletException {
        Account a = wallet.createAccount("Hans");
        Account b = wallet.getAccount(new AccountAlias("Hans"));
        assertEquals(a,b);
    }

    @Test
    public void testGetAccountByIndex() throws WalletException {
        Account a = wallet.createAccount("Hans");
        Account b = wallet.getAccount(new AccountIndex(0));
        assertEquals(a,b);
    }

    @Test
    public void testGetAccounts() throws WalletException {
        Account a = wallet.createAccount("Hans");
        Account b = wallet.createAccount("Billy");
        for (Account x : wallet.getAccounts())
            System.out.println(x);
    }

    @Test
    public void testBuildBasicOutput() throws WalletException {
        System.out.println(wallet.);
    }



}
