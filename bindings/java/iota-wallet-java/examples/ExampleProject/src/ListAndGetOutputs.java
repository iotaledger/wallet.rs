// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import org.iota.Wallet;
import org.iota.types.*;
import org.iota.types.account_methods.GetOutput;
import org.iota.types.account_methods.SyncAccount;
import org.iota.types.exceptions.WalletException;
import org.iota.types.ids.account.AccountAlias;
import org.iota.types.outputs.Output;
import org.iota.types.secret.MnemonicSecretManager;
public class ListAndGetOutputs {
    private static final String SHIMMER_TESTNET_NODE_URL = "https://api.testnet.shimmer.network";
    private static final int SHIMMER_COIN_TYPE = 4219;
    private static final String DEFAULT_DEVELOPMENT_MNEMONIC = "hidden enroll proud copper decide negative orient asset speed work dolphin atom unhappy game cannon scheme glow kid ring core name still twist actor";

    public static void main(String[] args) throws WalletException, InterruptedException {
        // Build the wallet.
        Wallet wallet = new Wallet(new WalletConfig()
                .withClientOptions(new ClientConfig().withNodes(new String[] { SHIMMER_TESTNET_NODE_URL }))
                .withSecretManager(new MnemonicSecretManager(DEFAULT_DEVELOPMENT_MNEMONIC))
                .withCoinType(SHIMMER_COIN_TYPE)
        );

        // Set up an account for this example.
        AccountHandle a = wallet.createAccount("Hans");

        // Check if the account is funded, else ask for funding.
        while(a.syncAccount(new SyncAccount()).getBaseCoin().getAvailable().equals("0")) {
            System.out.println("Please fund following address: " + a.getPublicAddresses()[0]);
            Thread.sleep(5000);
        }

        // Get outputs
        OutputData[] outputs = a.listOutputs(new org.iota.types.account_methods.ListOutputs());

        // Print outputs
        for(OutputData o : outputs)
            System.out.println(o);

        // Get a specific output by id
        OutputData o = a.getOutput(new GetOutput().withOutputId(outputs[0].getOutputId()));

        // Print the output
        System.out.println(o);
    }
}
