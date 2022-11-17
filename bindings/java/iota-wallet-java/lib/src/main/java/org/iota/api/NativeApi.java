// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.api;

import com.google.gson.Gson;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import org.apache.commons.lang3.SystemUtils;
import org.iota.types.WalletConfig;
import org.iota.types.exceptions.WalletException;
import org.iota.types.events.Event;
import org.iota.types.events.EventListener;
import org.iota.types.events.wallet.WalletEventType;

import java.io.IOException;

public class NativeApi {

    static {
        String libraryName = null;

        if (SystemUtils.IS_OS_LINUX)
            libraryName = "libiota_wallet.so";
        else if (SystemUtils.IS_OS_MAC)
            libraryName = "libiota_wallet.dylib";
        else if (SystemUtils.IS_OS_WINDOWS)
            libraryName = "iota_wallet.dll";
        else throw new RuntimeException("OS not supported");

        try {
            NativeUtils.loadLibraryFromJar("/" + libraryName);
        } catch (IOException e) {
            e.printStackTrace();
            throw new RuntimeException("cannot load native library");
        }

    }

    protected NativeApi(WalletConfig walletConfig) {
        createMessageHandler(new Gson().toJsonTree(walletConfig).toString());
    }

    protected static native void initLogger(String config);
    private static native void createMessageHandler(String config);

    // Destroys account handle
    protected static native void destroyHandle();

    private static native String sendMessage(String command);

    private static native String listen(String[] events, EventListener listener);

    private static JsonElement handleClientResponse(String methodName, String jsonResponse) throws WalletException {
        ClientResponse response = CustomGson.get().fromJson(jsonResponse, ClientResponse.class);

        switch (response.type) {
            case "panic":
                throw new RuntimeException(response.toString());
            case "error":
                throw new WalletException(methodName, response.payload.getAsJsonObject().toString());

            default:
                return response.payload;
        }
    }

    private static void handleCallback(String response, EventListener listener) throws WalletException {
        try {
            Event event = CustomGson.get().fromJson(response, Event.class);
            listener.receive(event);
        } catch (Exception e) {
            throw new WalletException("handleCallback", e.getMessage());
        }
    }

    public static void callListen(EventListener listener, WalletEventType... events) throws WalletException {
        String[] eventStrs = new String[events.length];
        for (int i = 0; i < events.length; i++) {
            eventStrs[i] = events[i].toString();
        }

        // Check for errors, no interest in result
        handleClientResponse("listen", listen(eventStrs, listener));
    }

    public static JsonElement callBaseApi(ClientCommand command) throws WalletException {
        //System.out.println("REQUEST: " + command);
        String jsonResponse = sendMessage(command.toString());
        //System.out.println("RESPONSE: " + jsonResponse);
        return handleClientResponse(command.methodName, jsonResponse);
    }

    private class ClientResponse {
        String type;
        JsonElement payload;
    }

    public static class ClientCommand {

        private String methodName;
        private JsonElement methodParams;

        public ClientCommand(String methodName) {
            this.methodName = methodName;
        }

        public ClientCommand(String methodName, JsonElement methodParams) {
            this.methodName = methodName;
            this.methodParams = methodParams;
        }

        @Override
        public String toString() {
            JsonObject message = new JsonObject();
            message.addProperty("cmd", methodName);
            message.add("payload", methodParams);

            return message.toString();
        }
    }
}