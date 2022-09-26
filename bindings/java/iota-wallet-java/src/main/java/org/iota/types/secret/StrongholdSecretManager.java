// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.secret;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonSerializationContext;
import com.google.gson.JsonSerializer;
import com.google.gson.annotations.JsonAdapter;

import java.lang.reflect.Type;
@JsonAdapter(StrongholdSecretManager.StrongholdSecretManagerAdapter.class)
public class StrongholdSecretManager extends SecretManager {

    private String password;
    private String timeout;
    private String snapshotPath;

    public StrongholdSecretManager(String password, String timeout, String snapshotPath) {
        this.password = password;
        this.timeout = timeout;
        this.snapshotPath = snapshotPath;
    }

    class StrongholdSecretManagerAdapter implements JsonSerializer<StrongholdSecretManager> {
        @Override
        public JsonElement serialize(StrongholdSecretManager s, Type typeOfSrc, JsonSerializationContext context) {
            JsonObject dto = new JsonObject();
            dto.addProperty("password", s.password);
            dto.addProperty("timeout", s.timeout);
            dto.addProperty("snapshotPath", s.snapshotPath);

            JsonObject o = new JsonObject();
            o.add("Stronghold", dto);

            return o;
        }
    }

}


