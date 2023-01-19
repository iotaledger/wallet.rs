import org.iota.Wallet;
import org.iota.external.logger.LevelFilter;
import org.iota.external.logger.LoggerOutputConfigBuilder;
import org.iota.types.*;
import org.iota.types.exceptions.InitializeWalletException;
import org.iota.types.exceptions.WalletException;
import org.iota.types.secret.StrongholdSecretManager;

public class Test {

    public static void main(String[] args) throws WalletException {

        // Initialise the logger for all debug output on Rusts' side.
        Wallet.initLogger(new LoggerOutputConfigBuilder().setLevelFilter(LevelFilter.Debug).setColorEnabled(true));

        // Build the wallet.
        Wallet wallet;

        try {
            wallet = new Wallet(new WalletConfig()
                    .withClientOptions(new ClientConfig().withNodes("https://api.shimmer.network"))
                    .withSecretManager(new StrongholdSecretManager("PASSWORD_FOR_ENCRYPTION", null, "example-wallet"))
                    .withCoinType(CoinType.Shimmer));
            //
            //wallet.storeMnemonic(Env.MNEMONIC);

            // wallet.setClientOptions(new ClientConfig().withNodes("https://api.testnet.shimmer.network"));

            for(AccountHandle f: wallet.getAccounts())  {
                System.out.println(f.getAlias());
                for(AccountAddress x: f.getAddresses())
                    System.out.println(x.getAddress());
            }
        } catch (InitializeWalletException e) {
            // TODO Auto-generated catch block
            e.printStackTrace();
        }

    }

}



