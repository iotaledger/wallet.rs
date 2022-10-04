// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import org.iota.Wallet;
import org.iota.types.*;
import org.iota.types.account_methods.SyncAccount;
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
        AccountHandle a = wallet.createAccount("Alice");

        // Sync account
        a.syncAccount(new SyncAccount().withOptions(new SyncOptions()));

        // Prepare the NFT transaction
        Transaction t = a.sendNft(new org.iota.types.account_methods.SendNft().withAddressesAndNftIds(new AddressAndNftId[] {
                new AddressAndNftId()
                        .withNftId(new NftId("0xdbed22679570aecc16da90648836607981e87c1ed3e3a24daf0942aa29a66003"))
                        .withAddress("rms1qpx0mcrqq7t6up73n4na0zgsuuy4p0767ut0qq67ngctj7pg4tm2ynsuynp")
        }));

        // Print NFT transaction
        System.out.println(t);
    }

}

