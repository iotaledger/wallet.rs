// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.secret;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonSerializationContext;
import com.google.gson.JsonSerializer;
import com.google.gson.annotations.JsonAdapter;

import java.lang.reflect.Type;

@JsonAdapter(SeedSecretManager.SeedSecretManagerAdapter.class)
public class SeedSecretManager extends SecretManager {

    private String hexSeed;

    public SeedSecretManager(String hexSeed) {
        this.hexSeed = hexSeed;
    }

    public static class SeedSecretManagerAdapter implements JsonSerializer<SeedSecretManager> {
        @Override
        public JsonElement serialize(SeedSecretManager s, Type typeOfSrc, JsonSerializationContext context) {
            JsonObject o = new JsonObject();
            o.addProperty("HexSeed", s.hexSeed);
            return o;
        }
    }

}


