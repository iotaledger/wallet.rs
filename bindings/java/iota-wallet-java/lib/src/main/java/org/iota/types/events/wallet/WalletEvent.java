// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.events.wallet;

import com.google.gson.*;
import com.google.gson.annotations.JsonAdapter;

import org.iota.api.CustomGson;
import org.iota.types.AbstractObject;
import org.iota.types.events.transactionprogress.TransactionProgressEvent;

import java.lang.reflect.Type;

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
            type = json.getAsJsonObject().entrySet().iterator().next().getKey();
        } else {
            // Enum without vaue
            type = json.getAsString();
            value = new JsonObject();
        }

        WalletEvent event;
        switch (type) {
            case "ConsolidationRequired": {
                event = CustomGson.get().fromJson("{}", ConsolidationRequired.class);
                break;
            }
            case "LedgerAddressGeneration": { // string Address
                event = CustomGson.get().fromJson(json, LedgerAddressGeneration.class);
                break;
            }
            case "NewOutput": { // NewOutputEvent
                event = CustomGson.get().fromJson(json, NewOutput.class);
                break;
            }
            case "SpentOutput": { // SpentOutputEvent
                event = CustomGson.get().fromJson(json, SpentOutput.class);
                break;
            }
            case "TransactionInclusion": { // TransactionInclusionEvent
                event = CustomGson.get().fromJson(json, TransactionInclusion.class);
                break;
            }
            case "TransactionProgress": {
                // Why doesnt it do this when fromJson on TransactionProgress??
                TransactionProgressEvent tEvent = CustomGson.get().fromJson(json, TransactionProgressEvent.class);
                event = tEvent;
                break;
            }
            default:
                throw new JsonParseException("unknown event type: " + type);
        }

        event.type = WalletEventType.valueOf(type);
        return event;
    }
}
