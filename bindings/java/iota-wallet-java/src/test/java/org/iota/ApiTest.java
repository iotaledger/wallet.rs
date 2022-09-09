package org.iota;

import org.iota.types.WalletConfig;
import org.iota.types.expections.WalletException;
import org.iota.types.secret.MnemonicSecretManager;
import org.junit.jupiter.api.AfterEach;

import static org.junit.jupiter.api.Assertions.assertTrue;

public class ApiTest {

    protected static final String DEFAULT_TESTNET_NODE_URL = "https://api.testnet.shimmer.network";
    protected static final String DEFAULT_TESTNET_FAUCET_URL = "https://faucet.testnet.shimmer.network/api/enqueue";
    protected static final String DEFAULT_DEVELOPMENT_MNEMONIC = "hidden enroll proud copper decide negative orient asset speed work dolphin atom unhappy game cannon scheme glow kid ring core name still twist actor";

    protected static Wallet wallet;
    protected static WalletConfig config = new WalletConfig()
            .withClientOptions(new WalletConfig.ClientConfig().withNodes(new String[] { DEFAULT_TESTNET_NODE_URL }))
            .withSecretManager(new MnemonicSecretManager(DEFAULT_DEVELOPMENT_MNEMONIC));

    static {
        wallet = new Wallet(config);
        try {
            assertTrue(wallet.getAccounts().length == 0);
        } catch (WalletException e) {
            throw new RuntimeException(e);
        }
    }

    @AfterEach
    protected void tearDown() throws WalletException {
        while(wallet.getAccounts().length > 0)
            wallet.removeLatestAccount();
        assertTrue(wallet.getAccounts().length == 0);
    }

}
