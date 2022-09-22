import org.iota.Wallet;
import org.iota.types.ClientConfig;
import org.iota.types.SyncOptions;
import org.iota.types.WalletConfig;
import org.iota.types.exceptions.WalletException;
import org.iota.types.secret.MnemonicSecretManager;

public class RecoverAccounts {
    private static final String SHIMMER_TESTNET_NODE_URL = "https://api.testnet.shimmer.network";
    private static final int SHIMMER_COIN_TYPE = 4219;
    private static final String DEFAULT_DEVELOPMENT_MNEMONIC = "hidden enroll proud copper decide negative orient asset speed work dolphin atom unhappy game cannon scheme glow kid ring core name still twist actor";

    public static void main(String[] args) throws WalletException {
        // Build the wallet
        Wallet wallet = new Wallet(new WalletConfig()
                .withClientOptions(new ClientConfig().withNodes(new String[] { SHIMMER_TESTNET_NODE_URL }))
                .withSecretManager(new MnemonicSecretManager(DEFAULT_DEVELOPMENT_MNEMONIC))
                .withCoinType(SHIMMER_COIN_TYPE)
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

        wallet.recoverAccounts(accountStartIndex,accountGapLimit,addressGapLimit, syncOptions);
    }
}
