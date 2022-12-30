// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import java.io.FileReader;
import java.io.IOException;
import org.iota.Wallet;
import org.iota.api.CustomGson;
import org.iota.types.*;
import org.iota.types.events.Event;
import org.iota.types.events.EventListener;
import org.iota.types.events.transaction.PreparedTransaction;
import org.iota.types.events.transaction.SelectingInputs;
import org.iota.types.events.transaction.TransactionProgressEvent;
import org.iota.types.events.wallet.WalletEvent;
import org.iota.types.exceptions.InitializeWalletException;
import org.iota.types.exceptions.WalletException;
import org.iota.types.secret.StrongholdSecretManager;

import com.google.gson.JsonElement;
import com.google.gson.JsonParser;

public class CallbackEvents {

    public static void main(String[] args) throws WalletException, InterruptedException, IOException, InitializeWalletException {
        // This example assumes that a wallet has already been created using the ´SetupWallet.java´ example.
        // If you haven't run the ´SetupWallet.java´ example yet, you must run it first to be able to load the wallet as shown below:
        Wallet wallet = new Wallet(new WalletConfig()
                .withClientOptions(new ClientConfig().withNodes("https://api.testnet.shimmer.network"))
                .withSecretManager(new StrongholdSecretManager(Env.STRONGHOLD_PASSWORD, null, Env.STRONGHOLD_VAULT_PATH))
                .withCoinType(CoinType.Shimmer)
                .withStoragePath(Env.STORAGE_PATH)
        );

        // Listen to all events.
        wallet.listen(new EventListener() {
            @Override
            public void receive(Event event) {
                System.out.println(
                        System.lineSeparator() + "Event receive: " + event.getEvent().getClass().getSimpleName());
                if (event.getEvent() instanceof TransactionProgressEvent) {
                    TransactionProgressEvent progress = (TransactionProgressEvent) event.getEvent();
                    System.out.println(progress.toString());
                } else {
                    System.out.println(event.getEvent().toString());
                }
            }
        });

        // Create a dummy event and trigger it to illustrate how the listener works.

        // Create the dummy event from JSON.
        JsonElement prepared = JsonParser.parseReader(
                new FileReader("src/res/prepared_transaction_data.json"));
        PreparedTransactionData data = CustomGson.get().fromJson(prepared, PreparedTransactionData.class);
        WalletEvent event = new PreparedTransaction(data);

        // Emit the dummy event.
        wallet.emitTestEvent(event);

        // Remove listeners when they are no longer needed. Listening to events will no longer be possible 
        // as all listeners have been removed.
        wallet.clearListeners();

        // Create another event.
        event = new SelectingInputs();

        // The second event is not received by our listener anymore because the listener has been removed from the wallet.
        wallet.emitTestEvent(event);

        // In case you are done and don't need the wallet instance anymore you can destroy the instance to clean up memory.
        // For this, check out the ´DestroyWallet.java´ example.
    }
}