// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.api;

import com.google.gson.Gson;
import com.google.gson.GsonBuilder;

public class GsonSingleton {

    private static Gson GSON = new GsonBuilder().serializeNulls().create();

    public static Gson getInstance() {
        return GSON;
    }
    
}