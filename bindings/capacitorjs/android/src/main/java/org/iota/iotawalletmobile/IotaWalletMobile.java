// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.iotawalletmobile;

import android.util.Log;

import com.getcapacitor.JSArray;
import com.getcapacitor.JSObject;
import com.getcapacitor.Plugin;
import com.getcapacitor.PluginCall;
import com.getcapacitor.PluginMethod;
import com.getcapacitor.annotation.CapacitorPlugin;

import java.util.Arrays;
import java.util.List;

import org.iota.Wallet;
import org.iota.api.WalletCommand;
import org.iota.types.ClientConfig;
import org.iota.types.CoinType;
import org.iota.types.WalletConfig;
import org.iota.types.events.Event;
import org.iota.types.events.EventListener;
import org.iota.types.events.transaction.TransactionProgressEvent;
import org.iota.types.events.wallet.WalletEventType;
import org.iota.types.exceptions.WalletException;
import org.iota.types.secret.StrongholdSecretManager;
import org.json.JSONException;
import org.json.JSONObject;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;

import static org.iota.api.NativeApi.callBaseApi;
import static org.iota.api.NativeApi.callListen;


@CapacitorPlugin(name = "IotaWalletMobile")
public class IotaWalletMobile extends Plugin {

    Wallet wallet = null;

    @PluginMethod()
    public void messageHandlerNew(final PluginCall call) throws JSONException {

        if (!call.getData().has("clientOptions")
                || !call.getData().has("storagePath")
                || !call.getData().has("coinType")
                || !call.getData().has("secretManager")) {
            call.reject("clientOptions, storagePath, coinType and secretManager are required");
        }
        JSObject _clientOptions = call.getObject("clientOptions");
        String _storagePath = call.getString("storagePath");
        Integer _coinType = call.getInt("coinType");
        JSObject _secretManager = call.getObject("secretManager");
        String path = getContext().getFilesDir() + _storagePath;
        Log.d("_clientOptions", _clientOptions.get("nodes").toString());


        JSONObject secretManager = _secretManager.getJSONObject("stronghold");
        Log.d("_storagePath", path + secretManager.get("snapshotPath"));
        if (_coinType == null) {
            return;
        }
        CoinType coinType = CoinType.Shimmer;
        if (CoinType.Iota.getCoinTypeValue() == _coinType) {
            coinType = CoinType.Iota;
        }

        try {
            wallet = new Wallet(new WalletConfig()
                    .withClientOptions(new ClientConfig().withNodes("https://api.testnet.shimmer.network"))
                    .withStoragePath(path)
                    .withSecretManager(
                            new StrongholdSecretManager(
                                    null,
                                    null,
//                                    path + secretManager.get("snapshotPath")
                                    path + "/wallet.stronghold"
                            )
                    )
                    .withCoinType(coinType)
            );
            JSObject ret = new JSObject();
            ret.put("messageHandler", 1);
            call.resolve(ret);
        } catch (Exception e) {
            e.printStackTrace();
        }
    }

    @PluginMethod()
    public void sendMessage(final PluginCall call) {
        try {
            if (!call.getData().has("message")) {
                call.reject("message are required");
            }
            String message = call.getString("message");
            if (message == null) {
                return;
            }

            JsonElement element = JsonParser.parseString(message);
            JsonObject jsonObject = element.getAsJsonObject();
            WalletCommand walletCommand;
            if (jsonObject.has("payload")) {
                walletCommand = new WalletCommand(
                        jsonObject.get("cmd").getAsString(),
                        jsonObject.get("payload")
                );
            }
            else {
                walletCommand = new WalletCommand(jsonObject.get("cmd").getAsString());

            }
            JsonElement jsonResponse = callBaseApi(walletCommand);
            JSObject ret = new JSObject();
            if (jsonResponse != null) {
                Log.d("jsonResponse", jsonResponse.toString());
                JsonObject clientResponse = new JsonObject();
                clientResponse.addProperty("type", jsonObject.get("cmd").getAsString());
                clientResponse.add("payload", jsonResponse);
                ret.put("result", clientResponse.toString());
            } else {
                ret.put("result", "Ok");
            }
            call.resolve(ret);
        } catch (Exception ex) {
            call.reject(ex.getMessage() + Arrays.toString(ex.getStackTrace()));
            Log.d("sendMessage Error", ex.getMessage() + Arrays.toString(ex.getStackTrace()));
        }
    }

    @PluginMethod(returnType = PluginMethod.RETURN_CALLBACK)
    public void listen(final PluginCall call) throws WalletException, JSONException {
        // if (!call.getData().has("eventTypes")) {
        //     call.reject("eventTypes and event are required");
        // }
        // JSArray eventTypes = call.getArray("eventTypes");
        // try {
        //     wallet.listen(new EventListener() {
        //         @Override
        //         public void receive(Event event) {
        //             JSObject walletResponse = new JSObject();
        //             walletResponse.put("result", event.getEvent().toString());
        //             call.resolve(walletResponse);
        //         }
        //     }, WalletEventType.valueOf(eventTypes.getString(0)));
        // } catch (WalletException e) {
        //     e.printStackTrace();
        // }
        // call.setKeepAlive(true);
        call.resolve();
    }


    @PluginMethod()
    public void destroy(final PluginCall call) {
        try {
            wallet.destroy();
            call.release(bridge);
        } catch (Exception ex) {
            call.reject(ex.getMessage() + Arrays.toString(ex.getStackTrace()));
        }
    }

    @PluginMethod()
    public void clearListeners(final PluginCall call) {
        try {
            wallet.clearListeners();
            call.release(bridge);
        } catch (Exception ex) {
            call.reject(ex.getMessage() + Arrays.toString(ex.getStackTrace()));
        }
    }

}
