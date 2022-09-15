// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.api;

import com.google.gson.Gson;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import org.apache.commons.lang3.SystemUtils;
import org.iota.types.WalletConfig;
import org.iota.types.exceptions.WalletException;

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

    private static native void createMessageHandler(String config);

    private static native String sendMessage(String command);

    protected JsonElement callBaseApi(ClientCommand command) throws WalletException {
        System.out.println("REQUEST: " + command);
        String jsonResponse = sendMessage(command.toString());
        System.out.println("RESPONSE: " + jsonResponse);
        ClientResponse response = new Gson().fromJson(jsonResponse, ClientResponse.class);

        switch (response.type) {
            case "Panic":
                throw new RuntimeException(response.toString());
            case "Error":
                throw new WalletException(command.methodName, response.payload.getAsJsonObject().toString());

            default:
                return response.payload;
        }
    }

    private class ClientResponse {
        String type;
        JsonElement payload;
    }

    protected static class ClientCommand {

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