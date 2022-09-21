import org.iota.Wallet;
import org.iota.types.AccountAddress;
import org.iota.types.account_methods.SyncAccount;
import org.iota.types.exceptions.WalletException;
import org.iota.types.ids.account.AccountAlias;

public class ExampleUtils {

    public static void setUpAccountWithFunds(Wallet wallet, String alias) throws WalletException, InterruptedException {
        AccountAddress address = wallet.createAccount(alias).getPublicAddresses()[0];
        while(wallet.syncAccount(new AccountAlias(alias), new SyncAccount()).getBaseCoin().getAvailable().equals("0")) {
            Thread.sleep(5000);
            System.out.println("Please fund following address: " + address);
        }
    }

}
