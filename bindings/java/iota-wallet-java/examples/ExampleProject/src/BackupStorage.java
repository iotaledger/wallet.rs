import org.iota.Wallet;
import org.iota.types.Account;
import org.iota.types.ClientConfig;
import org.iota.types.WalletConfig;
import org.iota.types.exceptions.WalletException;
import org.iota.types.secret.MnemonicSecretManager;

public class BackupStorage {
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

        // Backup wallet
        wallet.backup("./wallet_backup", "THE PASSWORD TO ENCRYPT THE BACKUP");
    }
}
