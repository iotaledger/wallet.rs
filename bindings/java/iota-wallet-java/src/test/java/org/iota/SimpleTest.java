package org.iota;

import org.iota.types.Account;
import org.iota.types.AccountAddress;
import org.iota.types.account_methods.ListAddresses;
import org.iota.types.account_methods.SetAlias;
import org.iota.types.exceptions.WalletException;
import org.iota.types.ids.account.AccountAlias;
import org.iota.types.ids.account.AccountIndex;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

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
        assertTrue(wallet.getAccounts().length == 2);
        for (Account x : wallet.getAccounts())
            System.out.println(x);
    }

    @Test
    public void testListAddresses() throws WalletException {
        Account a = wallet.createAccount("Hans");
        AccountAddress[] addresses = wallet.listAddresses(new AccountIndex(0), new ListAddresses());
        for(AccountAddress address : addresses) {
            System.out.println(address.getAddress());
        }
    }

    @Test
    public void testSetAlias() throws WalletException {
        wallet.createAccount("Hans");
        wallet.setAlias(new AccountIndex(0), new SetAlias().withAlias("Billy"));
        System.out.println(wallet.getAccount(new AccountIndex(0)).getAlias());
    }

}
