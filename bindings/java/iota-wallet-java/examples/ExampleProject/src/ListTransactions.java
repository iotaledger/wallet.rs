import org.iota.Wallet;
import org.iota.types.Account;
import org.iota.types.ClientConfig;
import org.iota.types.WalletConfig;
import org.iota.types.account_methods.SyncAccount;
import org.iota.types.exceptions.WalletException;
import org.iota.types.ids.account.AccountAlias;
import org.iota.types.payload.TransactionPayload;
import org.iota.types.secret.MnemonicSecretManager;

public class ListTransactions {
    private static final String SHIMMER_TESTNET_NODE_URL = "https://api.testnet.shimmer.network";
    private static final int SHIMMER_COIN_TYPE = 4219;
    private static final String DEFAULT_DEVELOPMENT_MNEMONIC = "hidden enroll proud copper decide negative orient asset speed work dolphin atom unhappy game cannon scheme glow kid ring core name still twist actor";

    public static void main(String[] args) throws WalletException, InterruptedException {

        // Build the wallet
        Wallet wallet = new Wallet(new WalletConfig()
                .withClientOptions(new ClientConfig().withNodes(new String[] { SHIMMER_TESTNET_NODE_URL }))
                .withSecretManager(new MnemonicSecretManager(DEFAULT_DEVELOPMENT_MNEMONIC))
                .withCoinType(SHIMMER_COIN_TYPE)
        );

        // Create an account
        Account firstAccount = wallet.createAccount("Hans");

        // Check if the account is funded, else ask for funding
        while(wallet.syncAccount(new AccountAlias("Hans"), new SyncAccount()).getBaseCoin().getAvailable().equals("0")) {
            Thread.sleep(5000);
            System.out.println("Please fund following address: " + firstAccount.getPublicAddresses()[0]);
        }

        // Get transactions
        TransactionPayload[] transactions = wallet.listTransactions(new AccountAlias(firstAccount.getAlias()), new org.iota.types.account_methods.ListTransactions());

        // Print transactions
        for(TransactionPayload t : transactions)
            System.out.println(t);

    }
}