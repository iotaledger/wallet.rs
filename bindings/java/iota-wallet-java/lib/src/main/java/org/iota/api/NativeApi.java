// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.api;

import com.google.gson.Gson;
import com.google.gson.JsonElement;
import org.apache.commons.lang3.SystemUtils;
import org.iota.types.WalletConfig;
import org.iota.types.exceptions.WalletException;

public class NativeApi {

    static {

        Throwable loadFromJavaPathThrowable = null;
        Throwable loadFromJarThrowable = null;

        try {
            loadFromJavaPath();
        } catch (Throwable t) {
            loadFromJavaPathThrowable = t;
        }

        if(loadFromJavaPathThrowable != null) {
            try {
                loadFromJar();
            } catch (Throwable t) {
                loadFromJarThrowable = t;
            }
        }

        if(loadFromJavaPathThrowable != null && loadFromJarThrowable != null) {
            loadFromJavaPathThrowable.printStackTrace();
            loadFromJarThrowable.printStackTrace();
            throw new RuntimeException("cannot load native library");
        }

    }

    private static void loadFromJavaPath() {
        System.loadLibrary("iota_wallet");
    }

    private static void loadFromJar() throws Throwable {
        String libraryName;

        if (SystemUtils.IS_OS_LINUX)
            libraryName = "libiota_wallet.so";
        else if (SystemUtils.IS_OS_MAC)
            libraryName = "libiota_wallet.dylib";
        else if (SystemUtils.IS_OS_WINDOWS)
            libraryName = "iota_wallet.dll";
        else throw new RuntimeException("OS not supported");

        NativeUtils.loadLibraryFromJar("/" + libraryName);
    }

    protected NativeApi(WalletConfig walletConfig) {
        createMessageHandler(new Gson().toJsonTree(walletConfig).toString());
    }

    private static native void createMessageHandler(String config);

    private static native String sendMessage(String command);

    public static JsonElement callBaseApi(WalletCommand command) throws WalletException {
        //System.out.println("REQUEST: " + command);
        String jsonResponse = sendMessage(command.toString());
        //System.out.println("RESPONSE: " + jsonResponse);
        WalletResponse response = CustomGson.get().fromJson(jsonResponse, WalletResponse.class);

        switch (response.type) {
            case "panic":
                throw new RuntimeException(response.toString());
            case "error":
                throw new WalletException(command.getMethodName(), response.payload.getAsJsonObject().toString());

            default:
                return response.payload;
        }
    }

    private class WalletResponse {
        String type;
        JsonElement payload;
    }

}