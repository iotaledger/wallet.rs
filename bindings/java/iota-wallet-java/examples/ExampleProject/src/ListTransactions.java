import org.iota.Wallet;
import org.iota.types.*;
import org.iota.types.account_methods.PrepareTransaction;
import org.iota.types.account_methods.SyncAccount;
import org.iota.types.exceptions.WalletException;
import org.iota.types.ids.OutputId;
import org.iota.types.ids.account.AccountAlias;
import org.iota.types.outputs.BasicOutput;
import org.iota.types.outputs.Output;
import org.iota.types.secret.MnemonicSecretManager;

import java.util.ArrayList;
import java.util.Collection;
import java.util.List;

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

        // Set up an account with funds for this example
        ExampleUtils.setUpAccountWithFunds(wallet, "Hans");

        // Sync account
        wallet.syncAccount(new AccountAlias("Hans"), new SyncAccount());

        // List transactions
        Transaction[] transactions = wallet.listTransactions(new AccountAlias("Hans"));

        // Print transactions
        for (Transaction tx : transactions) {
            System.out.println(tx.toString());
        }

    }

}

