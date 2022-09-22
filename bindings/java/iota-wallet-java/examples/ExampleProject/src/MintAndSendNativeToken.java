import org.iota.Wallet;
import org.iota.types.*;
import org.iota.types.account_methods.MintNativeToken;
import org.iota.types.account_methods.MintNfts;
import org.iota.types.account_methods.SendNativeTokens;
import org.iota.types.exceptions.WalletException;
import org.iota.types.ids.account.AccountAlias;
import org.iota.types.secret.MnemonicSecretManager;

public class MintAndSendNativeToken {
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

        // Set up an account for this example.
        ExampleUtils.setUpAccountWithFunds(wallet, "Hans");

        // Configure the Native Token
        NativeTokenOptions options = new NativeTokenOptions();
        options.withCirculatingSupply("0x17");
        options.withMaximumSupply("0x64");

        // Mint the Native Token
        MintTokenTransaction t = wallet.mintNativeToken(new AccountAlias("Hans"), new MintNativeToken().withNativeTokenOptions(options));

        // Print Native Token
        System.out.println(t);
    }

}

