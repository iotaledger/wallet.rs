// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import java.io.FileReader;
import java.io.IOException;
import java.nio.charset.Charset;
import java.nio.file.Files;
import java.nio.file.Paths;

import org.iota.Wallet;
import org.iota.api.CustomGson;
import org.iota.api.NativeApi;
import org.iota.types.*;
import org.iota.types.ClientConfig.ApiTimeout;
import org.iota.types.account_methods.GenerateAddresses;
import org.iota.types.account_methods.SyncAccount;
import org.iota.types.events.Event;
import org.iota.types.events.EventListener;
import org.iota.types.events.transactionprogress.TransactionProgressEvent;
import org.iota.types.events.wallet.WalletEventType;
import org.iota.types.exceptions.WalletException;
import org.iota.types.ids.account.AccountAlias;
import org.iota.types.secret.StrongholdSecretManager;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;

public class CallbackEvents {


    public static void main(String[] args) throws WalletException, InterruptedException, IOException {
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

        // Listen to all events. An empty array also indicates all events
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
        }, WalletEventType.values());

        // Read in a prepared transaction stored as JSON to be used in this example.
        JsonElement prepared = JsonParser.parseReader(
                new FileReader("src/main/res/prepared_transaction_data.json"));

        // Create the dummy transaction event
        JsonObject transactionEvent = new JsonObject();
        transactionEvent.add("PreparedTransaction", prepared);

        // Create The Wallet event with the transaction event
        JsonObject walletEvent = new JsonObject();
        walletEvent.add("TransactionProgress", transactionEvent);

        // Emit the fake event
        wallet.emitTestEvent(walletEvent);

        // Clear listeners
        wallet.clearListeners();

        // Create another event
        JsonObject selectingInputsEvent = new JsonObject();
        selectingInputsEvent.addProperty("TransactionProgress", "SelectingInputs");

        // Emitted event is not received by our listener because the listener has
        // already been unsubscribed from the wallet.
        wallet.emitTestEvent(selectingInputsEvent);

        wallet.destroy();
    }
}