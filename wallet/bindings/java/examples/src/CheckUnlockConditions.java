// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import org.iota.Wallet;
import org.iota.types.*;
import org.iota.types.account_methods.PrepareOutput;
import org.iota.types.account_methods.SyncAccount;
import org.iota.types.addresses.Ed25519Address;
import org.iota.types.exceptions.InitializeWalletException;
import org.iota.types.exceptions.WalletException;
import org.iota.types.outputs.Output;
import org.iota.types.outputs.BasicOutput;
import org.iota.types.unlock_conditions.AddressUnlockCondition;
import org.iota.types.ids.account.AccountAlias;
import org.iota.types.secret.StrongholdSecretManager;
import java.util.ArrayList;

public class CheckUnlockConditions {
    public static void main(String[] args) throws WalletException, InterruptedException, InitializeWalletException {
        // This example assumes that a wallet has already been created using the ´SetupWallet.java´ example.
        // If you haven't run the ´SetupWallet.java´ example yet, you must run it first to be able to load the wallet as shown below:
        Wallet wallet = new Wallet(new WalletConfig()
            .withClientOptions(new ClientConfig().withNodes(Env.NODE))
            .withSecretManager(new StrongholdSecretManager(Env.STRONGHOLD_PASSWORD, null, Env.STRONGHOLD_VAULT_PATH))
            .withCoinType(CoinType.Shimmer)
            .withStoragePath(Env.STORAGE_PATH)
        );

        // Get account.
        AccountHandle a = wallet.getAccount(new AccountAlias(Env.ACCOUNT_NAME));

        // Convert addresses to hex.
        ArrayList<String> hexEncodedAccountAddresses = new ArrayList<String>();
        for (AccountAddress address: a.getAddresses())
            hexEncodedAccountAddresses.add(wallet.bech32ToHex(address.getAddress()));

        // Build an output.
        Output output = a.prepareOutput(new PrepareOutput().withOptions(new OutputOptions()
            .withAmount("1000000")
            .withRecipientAddress(a.getPublicAddresses()[0].getAddress())
        ));

        boolean controlledByAccount = false;

        BasicOutput basicOutput = (BasicOutput) output;
        AddressUnlockCondition addressUnlockCondition = (AddressUnlockCondition) basicOutput.getUnlockConditions()[0];
        Ed25519Address ed25519Address = (Ed25519Address) addressUnlockCondition.getAddress();
        if (basicOutput.getUnlockConditions().length == 1 &&
            addressUnlockCondition.getType() == 0 &&
            hexEncodedAccountAddresses.contains(ed25519Address.getPubKeyHash())) {
            controlledByAccount = true;
        }

        // Print the balance.
        System.out.println("The output has only an address unlock condition and that its address is from the account: " + controlledByAccount);

        // In case you are done and don't need the wallet instance anymore you can destroy the instance to clean up memory.
        // For this, check out the ´DestroyWallet.java´ example.
    }
}