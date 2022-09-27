// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import org.iota.Wallet;
import org.iota.types.*;
import org.iota.types.exceptions.WalletException;
import org.iota.types.ids.NftId;
import org.iota.types.secret.StrongholdSecretManager;

public class SendNft {
    private static final String DEFAULT_DEVELOPMENT_MNEMONIC = "hidden enroll proud copper decide negative orient asset speed work dolphin atom unhappy game cannon scheme glow kid ring core name still twist actor";

    public static void main(String[] args) throws WalletException, InterruptedException {
        // Build the wallet.
        Wallet wallet = new Wallet(new WalletConfig()
                .withClientOptions(new ClientConfig().withNodes("https://api.testnet.shimmer.network"))
                .withSecretManager(new StrongholdSecretManager("PASSWORD_FOR_ENCRYPTION", null, "example-wallet"))
                .withCoinType(CoinType.Shimmer)
        );
        wallet.storeMnemonic(DEFAULT_DEVELOPMENT_MNEMONIC);

        // Set up an account for this example.
        AccountHandle a = wallet.createAccount("Hans");

        // Prepare the NFT transaction

        Transaction t = a.sendNft(new org.iota.types.account_methods.SendNft().withAddressesAndNftIds(new AddressAndNftId[] {
                new AddressAndNftId()
                        .withNftId(new NftId("0xd96c9cdf8c6b095f7ab44105f8298c766a7433664db651bfbd7832566d59f1030000"))
                        .withAddress("rms1qpszqzadsym6wpppd6z037dvlejmjuke7s24hm95s9fg9vpua7vluaw60xu")
        }));

        // Print NFT transaction
        System.out.println(t);
    }

}

