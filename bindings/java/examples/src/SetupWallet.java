// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import org.iota.Wallet;
import org.iota.external.logger.LevelFilter;
import org.iota.external.logger.LoggerOutputConfigBuilder;
import org.iota.types.ClientConfig;
import org.iota.types.CoinType;
import org.iota.types.WalletConfig;
import org.iota.types.exceptions.InitializeWalletException;
import org.iota.types.exceptions.WalletException;
import org.iota.types.secret.StrongholdSecretManager;

public class SetupWallet {

    private static final String DEFAULT_DEVELOPMENT_MNEMONIC = "hidden enroll proud copper decide negative orient asset speed work dolphin atom unhappy game cannon scheme glow kid ring core name still twist actor";

    public static void main(String[] args) throws WalletException, InterruptedException, InitializeWalletException {
        // Initialise the logger for all debug output on Rusts' side.
        Wallet.initLogger(new LoggerOutputConfigBuilder().setLevelFilter(LevelFilter.Debug).setColorEnabled(true));

        // Setup and store the wallet.
        Wallet wallet = new Wallet(new WalletConfig()
                .withClientOptions(new ClientConfig().withNodes("https://api.testnet.shimmer.network"))
                .withSecretManager(new StrongholdSecretManager(Env.STRONGHOLD_PASSWORD, null, Env.STORAGE_PATH + Env.STRONGHOLD_SNAPSHOT_NAME))
                .withCoinType(CoinType.Shimmer)
                .withStoragePath(Env.STORAGE_PATH)
        );
        wallet.storeMnemonic(DEFAULT_DEVELOPMENT_MNEMONIC);
    }
}
