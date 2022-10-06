// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types;

import com.google.gson.*;
import com.google.gson.annotations.JsonAdapter;

import java.lang.reflect.Type;

@JsonAdapter(BaseCoinBalanceAdapter.class)
public class BaseCoinBalance extends AbstractObject {
    /// Total amount
    private int total;
    /// Balance that can currently be spent
    private int available;

    public BaseCoinBalance(int total, int available) {
        this.total = total;
        this.available = available;
    }

    public int getTotal() {
        return total;
    }

    public int getAvailable() {
        return available;
    }
}

class BaseCoinBalanceAdapter implements JsonDeserializer<BaseCoinBalance>, JsonSerializer<BaseCoinBalance> {
    @Override
    public BaseCoinBalance deserialize(final JsonElement json, final Type typeOfT, final JsonDeserializationContext context)
            throws JsonParseException {

        int total = Integer.parseInt(json.getAsJsonObject().get("total").getAsString());
        int available = Integer.parseInt(json.getAsJsonObject().get("available").getAsString());

        return new BaseCoinBalance(total, available);
    }

    public JsonElement serialize(BaseCoinBalance src, Type typeOfSrc, JsonSerializationContext context) {
        JsonObject o = new JsonObject();
        o.addProperty("total", String.valueOf(src.getTotal()));
        o.addProperty("available", String.valueOf(src.getAvailable()));
        return o;
    }
}