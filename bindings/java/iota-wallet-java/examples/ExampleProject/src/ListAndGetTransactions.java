import org.iota.Wallet;
import org.iota.types.*;
import org.iota.types.account_methods.GetTransaction;
import org.iota.types.exceptions.WalletException;
import org.iota.types.ids.account.AccountAlias;
import org.iota.types.payload.TransactionPayload;
import org.iota.types.secret.MnemonicSecretManager;

public class ListAndGetTransactions {
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

        // Set up a transaction for this example.
        AccountAddress address = wallet.getAccount(new AccountAlias("Hans")).getPublicAddresses()[0];
        wallet.sendAmount(new AccountAlias("Hans"), new org.iota.types.account_methods.SendAmount().withAddressesWithAmount(
                new AddressWithAmount[] { new AddressWithAmount().withAddress(address.getAddress()).withAmount("1000000")}
        ));

        // List transactions
        Transaction[] transactions = wallet.listTransactions(new AccountAlias("Hans"));

        // Print transactions
        for (Transaction tx : transactions)
            System.out.println(tx.toString());

        // Get a specific transaction
        Transaction transaction = wallet.getTransaction(new AccountAlias("Hans"), new GetTransaction().withTransactionId(transactions[0].getTransactionId()));

        // Print transaction
        System.out.println(transaction);
    }

}

