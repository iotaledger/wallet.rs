// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import java.io.FileReader;
import java.io.IOException;
import org.iota.Wallet;
import org.iota.types.*;
import org.iota.types.ClientConfig.ApiTimeout;
import org.iota.types.events.Event;
import org.iota.types.events.EventListener;
import org.iota.types.events.transaction.TransactionProgressEvent;
import org.iota.types.exceptions.InitializeWalletException;
import org.iota.types.exceptions.WalletException;
import org.iota.types.secret.StrongholdSecretManager;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;

public class CallbackEvents {

    public static void main(String[] args)
            throws WalletException, InterruptedException, IOException, InitializeWalletException {
        // This example assumes that a wallet has already been created using the
        // ´CreateWallet.java´ example.
        // If you have not run the ´CreateAccount.java´ example yet, run it first to
        // ensure that the wallet can be loaded correctly.
        Wallet wallet = new Wallet(new WalletConfig()
                .withClientOptions(
                        new ClientConfig().withApiTimeout(new ApiTimeout().withSecs(60)).withNodes(Env.NODE))
                .withSecretManager(
                        new StrongholdSecretManager(Env.STRONGHOLD_PASSWORD, null, Env.STRONGHOLD_SNAPSHOT_PATH))
                .withCoinType(CoinType.Shimmer));

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
        JsonObject transactionEvent = new JsonObject();
        transactionEvent.add("PreparedTransaction", prepared);
        JsonObject walletEvent = new JsonObject();
        walletEvent.add("TransactionProgress", transactionEvent);

        // Emit the dummy event.
        wallet.emitTestEvent(walletEvent);

        // Remove listeners when they are no longer needed. Listening to events will no longer be possible as all listeners have been removed.
        wallet.clearListeners();

        // Create another dummy event.
        JsonObject selectingInputsEvent = new JsonObject();
        selectingInputsEvent.addProperty("TransactionProgress", "SelectingInputs");

        // Emitted event is not received by our listener because the listener has
        // already been removed from the wallet.
        wallet.emitTestEvent(selectingInputsEvent);
    }
}