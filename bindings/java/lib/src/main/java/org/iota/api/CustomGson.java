// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.api;

import com.google.gson.Gson;
import com.google.gson.GsonBuilder;

public class CustomGson {

    private static Gson GSON = new GsonBuilder().serializeNulls().create();

    public static Gson get() {
        return GSON;
    }
    
}