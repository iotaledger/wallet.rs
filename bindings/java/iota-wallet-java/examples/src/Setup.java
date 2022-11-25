// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import org.iota.Wallet;
import org.iota.external.logger.LevelFilter;
import org.iota.external.logger.LoggerOutputConfigBuilder;
import org.iota.types.*;
import org.iota.types.exceptions.InitializeWalletException;
import org.iota.types.exceptions.WalletException;
import org.iota.types.secret.StrongholdSecretManager;

public class Setup {

    public static void main(String[] args) throws WalletException, InterruptedException, InitializeWalletException {

        // Initialise the logger for all debug output on rusts' side
        Wallet.initLogger(new LoggerOutputConfigBuilder().setLevelFilter(LevelFilter.Debug).setColorEnabled(true));

        // Set up a wallet instance
        Wallet wallet = new Wallet(new WalletConfig()
                .withClientOptions(new ClientConfig().withNodes(Env.NODE))
                .withSecretManager(
                        new StrongholdSecretManager(Env.STRONGHOLD_PASSWORD, null, Env.STRONGHOLD_SNAPSHOT_PATH))
                .withCoinType(CoinType.Shimmer));

        // We call a wallet method here on the valid instance
        String mnemonic = wallet.generateMnemonic();
        System.out.println(mnemonic);

        // Destroy wallet instance, cleaning up memory
        wallet.destroy();

        try {
            // Calling this will throw an error as the instance is cleaned up
            wallet.generateMnemonic();
        } catch (NullPointerException e) {
            // Will print "Wallet not initialised"
            System.out.println(e.getMessage());
        }
    }
}
