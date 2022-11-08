// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import org.iota.Wallet;
import org.iota.types.*;
import org.iota.types.account_methods.GenerateAddresses;
import org.iota.types.account_methods.SyncAccount;
import org.iota.types.events.Event;
import org.iota.types.events.EventListener;
import org.iota.types.events.transactionprogress.TransactionProgressEvent;
import org.iota.types.events.wallet.TransactionProgress;
import org.iota.types.events.wallet.WalletEventType;
import org.iota.types.exceptions.WalletException;
import org.iota.types.ids.account.AccountAlias;
import org.iota.types.secret.StrongholdSecretManager;

public class CallbackEvents {

    static class Callback implements EventListener {
        @Override
        public void receive(Event event) {
            System.out.println("ConsolidationRequired receive: ");
            System.out.println(event.getEvent());
        }
    }

    public static void main(String[] args) throws WalletException, InterruptedException {
        // This example assumes that a wallet has already been created using the
        // ´CreateWallet.java´ example.
        // If you have not run the ´CreateAccount.java´ example yet, run it first to
        // ensure that the wallet can be loaded correctly.
        Wallet wallet = new Wallet(new WalletConfig()
                .withClientOptions(new ClientConfig().withNodes(Env.NODE))
                .withSecretManager(new StrongholdSecretManager(Env.PASSWORD, null, Env.SNAPSHOT_PATH))
                .withCoinType(CoinType.Shimmer));

        Long id = wallet.listen(new EventListener() {

            @Override
            public void receive(Event event) {
                System.out.println("TransactionProgress receive: " + event.getEvent());
                if (event.getEvent() instanceof TransactionProgress) {
                    TransactionProgress progress = (TransactionProgress) event.getEvent();
                    System.out.println("type: " + progress.getClass().getName());
                }
            }

        }, WalletEventType.TransactionProgress);
        long id2 = wallet.listen(new Callback(), WalletEventType.ConsolidationRequired);

        // Get account and sync it with the registered node to ensure that its balances
        // are up-to-date.
        AccountHandle account = wallet.getAccount(new AccountAlias(Env.ACCOUNT_NAME));

        // Generate two addresses.
        AccountAddress[] addresses = account.generateAddresses(new GenerateAddresses().withAmount(5));

        AccountBalance balance = account.syncAccount(new SyncAccount().withOptions(new SyncOptions()));

        // Fund the account for this example.
        // ExampleUtils.fundAccount(account);

        // TODO: replace with your own values.
        String receiverAddress = account.getPublicAddresses()[0].getAddress();
        String amount = "1000000";

        // Send transaction.
        Transaction t = account.sendAmount(
                new org.iota.types.account_methods.SendAmount().withAddressesWithAmount(new AddressWithAmount[] {
                        new AddressWithAmount()
                                .withAddress(receiverAddress)
                                .withAmount(amount)
                }));

        System.out.println(
                "Transaction: " + t.getTransactionId() + " Block sent: " + Env.NODE + "/api/core/v2/blocks/" +
                        t.getBlockId());
    }
}