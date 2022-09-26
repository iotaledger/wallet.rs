// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import org.iota.Wallet;
import org.iota.types.*;
import org.iota.types.account_methods.MintNfts;
import org.iota.types.exceptions.WalletException;
import org.iota.types.ids.account.AccountAlias;
import org.iota.types.secret.MnemonicSecretManager;
import org.iota.types.secret.StrongholdSecretManager;

public class MintAndSendNft {
    private static final String SHIMMER_TESTNET_NODE_URL = "https://api.testnet.shimmer.network";
    private static final int SHIMMER_COIN_TYPE = 4219;
    private static final String DEFAULT_DEVELOPMENT_MNEMONIC = "hidden enroll proud copper decide negative orient asset speed work dolphin atom unhappy game cannon scheme glow kid ring core name still twist actor";

    public static void main(String[] args) throws WalletException, InterruptedException {
        // Build the wallet.
        Wallet wallet = new Wallet(new WalletConfig()
                .withClientOptions(new ClientConfig().withNodes(SHIMMER_TESTNET_NODE_URL))
                .withSecretManager(new StrongholdSecretManager("PASSWORD_FOR_ENCRYPTION", 5, "example-wallet"))
                .withCoinType(SHIMMER_COIN_TYPE)
        );
        wallet.storeMnemonic(DEFAULT_DEVELOPMENT_MNEMONIC);

        // Set up an account for this example.
        AccountHandle a = ExampleUtils.setUpAccountWithFunds(wallet, "Hans");

        // Configure the NFT
        NftOptions options = new NftOptions();
        options.withMetadata("0x5368696d6d65722e20546f6b656e697a652045766572797468696e672e2048656c6c6f2066726f6d20746865204a6176612062696e64696e672e");

        // Mint the NFT
        Transaction nftTransaction = a.mintNfts(new MintNfts().withNftsOptions(new NftOptions[]{ options }));

        // Print NFT transaction
        System.out.println(nftTransaction);
    }

}

