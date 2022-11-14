// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.events.wallet;

import com.google.gson.*;
import com.google.gson.annotations.JsonAdapter;

import org.iota.api.CustomGson;
import org.iota.types.AbstractObject;
import org.iota.types.events.transactionprogress.TransactionProgressEvent;

import java.lang.reflect.Type;
import java.util.Map.Entry;

@JsonAdapter(WalletEventAdapter.class)
public abstract class WalletEvent extends AbstractObject {

    protected WalletEventType type;

    public WalletEventType getType() {
        return type;
    }
}

class WalletEventAdapter implements JsonDeserializer<WalletEvent> {
    @Override
    public WalletEvent deserialize(final JsonElement json, final Type typeOfT, final JsonDeserializationContext context)
            throws JsonParseException {

        String type;
        JsonElement value;

        // Ennum with value
        if (json.isJsonObject()) {
            Entry<String, JsonElement> entry = json.getAsJsonObject().entrySet().iterator().next();
            type = entry.getKey();
            value = entry.getValue();
        } else {
            // Enum without vaue
            type = json.getAsString();
            value = new JsonObject();
        }

        WalletEvent event;
        switch (type) {
            case "ConsolidationRequired": {
                event = CustomGson.get().fromJson(value, ConsolidationRequired.class);
                break;
            }
            case "LedgerAddressGeneration": {
                event = CustomGson.get().fromJson(value, LedgerAddressGeneration.class);
                break;
            }
            case "NewOutput": { // NewOutputEvent
                event = CustomGson.get().fromJson(value, NewOutput.class);
                break;
            }
            case "SpentOutput": { // SpentOutputEvent
                event = CustomGson.get().fromJson(value, SpentOutput.class);
                break;
            }
            case "TransactionInclusion": {
                event = CustomGson.get().fromJson(value, TransactionInclusion.class);
                break;
            }
            case "TransactionProgress": {
                // Needs the whole object to deterine enum type
                event = CustomGson.get().fromJson(json, TransactionProgressEvent.class);
                break;
            }
            default:
                throw new JsonParseException("unknown wallet event type: " + type);
        }

        event.type = WalletEventType.valueOf(type);
        return event;
    }
}
