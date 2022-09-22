import org.iota.Wallet;
import org.iota.types.AccountBalance;
import org.iota.types.AccountHandle;
import org.iota.types.ClientConfig;
import org.iota.types.WalletConfig;
import org.iota.types.account_methods.SyncAccount;
import org.iota.types.exceptions.WalletException;
import org.iota.types.ids.account.AccountAlias;
import org.iota.types.secret.MnemonicSecretManager;

public class GetBalance {

    private static final String SHIMMER_TESTNET_NODE_URL = "https://api.testnet.shimmer.network";
    private static final int SHIMMER_COIN_TYPE = 4219;
    private static final String DEFAULT_DEVELOPMENT_MNEMONIC = "hidden enroll proud copper decide negative orient asset speed work dolphin atom unhappy game cannon scheme glow kid ring core name still twist actor";


    public static void main(String[] args) throws WalletException, InterruptedException {
        // Build the wallet.
        Wallet wallet = new Wallet(new WalletConfig()
                .withClientOptions(new ClientConfig().withNodes(new String[] { SHIMMER_TESTNET_NODE_URL }))
                .withSecretManager(new MnemonicSecretManager(DEFAULT_DEVELOPMENT_MNEMONIC))
                .withCoinType(SHIMMER_COIN_TYPE)
        );

        // Create an account.
        AccountHandle a = wallet.createAccount("Hans");

        // Sync account with the registered node to make sure the correct balance is returned.
        AccountBalance balance = a.syncAccount(new SyncAccount());

        // Print the balance.
        System.out.println(balance.getBaseCoin().getTotal());

    }
}
