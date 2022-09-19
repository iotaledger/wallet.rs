// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.secret;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonSerializationContext;
import com.google.gson.JsonSerializer;
import com.google.gson.annotations.JsonAdapter;

import java.lang.reflect.Type;

@JsonAdapter(LedgerNanoSecretManager.LedgerNanoSecretManagerAdapter.class)
public class LedgerNanoSecretManager extends SecretManager {
    private boolean isSimulator;

    public LedgerNanoSecretManager(boolean isSimulator) {
        this.isSimulator = isSimulator;
    }

    public boolean isSimulator() {
        return isSimulator;
    }

    class LedgerNanoSecretManagerAdapter implements JsonSerializer<LedgerNanoSecretManager> {
        @Override
        public JsonElement serialize(LedgerNanoSecretManager src, Type typeOfSrc, JsonSerializationContext context) {
            JsonObject o = new JsonObject();
            o.addProperty("LedgerNano", src.isSimulator());
            return o;
        }
    }

}
