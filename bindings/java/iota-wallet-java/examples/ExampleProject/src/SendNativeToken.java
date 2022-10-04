// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import org.iota.Wallet;
import org.iota.types.*;
import org.iota.types.account_methods.SendNativeTokens;
import org.iota.types.account_methods.SyncAccount;
import org.iota.types.exceptions.WalletException;
import org.iota.types.ids.TokenId;
import org.iota.types.secret.StrongholdSecretManager;

import java.util.HashMap;
import java.util.Map;

public class SendNativeToken {
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
        AccountHandle a = ExampleUtils.setUpAccountWithFunds(wallet, "Alice");

        // Sync account
        a.syncAccount(new SyncAccount().withOptions(new SyncOptions()));

        // Get a tokenId from your account balance after running example
        // 22-mint-native-tokens.js
        TokenId tokenId = new TokenId("0x08429fe5864378ce70699fc2d22bb144cb86a3c4833d136e3b95c5dadfd6ba0cef0300000000");
        // `100` hex encoded
        String tokenAmount = "0x17";

        HashMap<TokenId, String> nativeToken = new HashMap<>();
        nativeToken.put(tokenId, tokenAmount);

        Transaction t = a.sendNativeTokens(new SendNativeTokens().withAddressesNativeTokens(new AddressNativeTokens[]{ new AddressNativeTokens()
                //TODO: Replace with the address of your choice!
                .withAddress("rms1qpx0mcrqq7t6up73n4na0zgsuuy4p0767ut0qq67ngctj7pg4tm2ynsuynp")
                .withNativeTokens(new AddressNativeTokens.NativeTokenTuple[] {
                        new AddressNativeTokens.NativeTokenTuple(tokenId, tokenAmount)
                })
        }));

        // Print Native Token transaction
        System.out.println(t);
    }

}

