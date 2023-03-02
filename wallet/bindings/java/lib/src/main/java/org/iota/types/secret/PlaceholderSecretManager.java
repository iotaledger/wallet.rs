// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.secret;

import com.google.gson.JsonElement;
import com.google.gson.JsonPrimitive;
import com.google.gson.JsonSerializationContext;
import com.google.gson.JsonSerializer;
import com.google.gson.annotations.JsonAdapter;

import java.lang.reflect.Type;
@JsonAdapter(PlaceholderSecretManagerAdapter.class)
public class PlaceholderSecretManager extends SecretManager {}
class PlaceholderSecretManagerAdapter implements JsonSerializer<PlaceholderSecretManager> {
    @Override
    public JsonElement serialize(PlaceholderSecretManager s, Type typeOfSrc, JsonSerializationContext context) {
        return new JsonPrimitive("Placeholder");
    }
}