// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types;

import com.google.gson.*;
import com.google.gson.annotations.JsonAdapter;
import org.iota.types.ids.OutputId;
import org.iota.types.payload.TaggedDataPayload;

import java.lang.reflect.Type;

public class TransactionOptions extends AbstractObject {

    private RemainderValueStrategy remainderValueStrategy = new ReuseAddress();
    private TaggedDataPayload taggedDataPayload;
    private OutputId[] customInputs;
    private boolean allowBurning;
    private String note;

    public RemainderValueStrategy getRemainderValueStrategy() {
        return remainderValueStrategy;
    }

    public TransactionOptions withRemainderValueStrategy(RemainderValueStrategy remainderValueStrategy) {
        this.remainderValueStrategy = remainderValueStrategy;
        return this;
    }

    public TaggedDataPayload getTaggedDataPayload() {
        return taggedDataPayload;
    }

    public TransactionOptions withTaggedDataPayload(TaggedDataPayload taggedDataPayload) {
        this.taggedDataPayload = taggedDataPayload;
        return this;
    }

    public OutputId[] getCustomInputs() {
        return customInputs;
    }

    public TransactionOptions withCustomInputs(OutputId[] customInputs) {
        this.customInputs = customInputs;
        return this;
    }

    public boolean isAllowBurning() {
        return allowBurning;
    }

    public TransactionOptions withAllowBurning(boolean allowBurning) {
        this.allowBurning = allowBurning;
        return this;
    }

    public String getNote() {
        return note;
    }

    public TransactionOptions withNote(String note) {
        this.note = note;
        return this;
    }

    @JsonAdapter(RemainderValueStrategyAdapter.class)
    public static abstract class RemainderValueStrategy {}

    class RemainderValueStrategyAdapter implements JsonSerializer<RemainderValueStrategy> {
        @Override
        public JsonElement serialize(RemainderValueStrategy src, Type typeOfSrc, JsonSerializationContext context) {
            if(src instanceof ReuseAddress) {
                return new Gson().toJsonTree(src, ReuseAddress.class);
            } else if(src instanceof ChangeAddress) {
                return new Gson().toJsonTree(src, ChangeAddress.class);
            } else if(src instanceof CustomAddress) {
                return new Gson().toJsonTree(src, CustomAddress.class);
            } else {
                throw new JsonParseException("unknown type: " + src.getClass().getSimpleName());
            }
        }
    }

    @JsonAdapter(ReuseAddressAdapter.class)
    public static class ReuseAddress extends RemainderValueStrategy {}
    @JsonAdapter(ChangeAddressAdapter.class)
    public static class ChangeAddress extends RemainderValueStrategy {}
    @JsonAdapter(CustomAddressAdapter.class)
    public static class CustomAddress extends RemainderValueStrategy {
        private AccountAddress content;

        public CustomAddress(AccountAddress content) {
            this.content = content;
        }

        public AccountAddress getContent() {
            return content;
        }
    }

    class ReuseAddressAdapter implements JsonSerializer<ReuseAddress> {
        @Override
        public JsonElement serialize(ReuseAddress src, Type typeOfSrc, JsonSerializationContext context) {
            JsonObject o = new JsonObject();
            o.addProperty("strategy", src.getClass().getSimpleName());
            return o;
        }
    }

    class ChangeAddressAdapter implements JsonSerializer<ChangeAddress> {
        @Override
        public JsonElement serialize(ChangeAddress src, Type typeOfSrc, JsonSerializationContext context) {
            JsonObject o = new JsonObject();
            o.addProperty("strategy", src.getClass().getSimpleName());
            return o;
        }
    }

    class CustomAddressAdapter implements JsonSerializer<CustomAddress> {
        @Override
        public JsonElement serialize(CustomAddress src, Type typeOfSrc, JsonSerializationContext context) {
            JsonObject o = new JsonObject();
            o.addProperty("strategy", src.getClass().getSimpleName());
            o.addProperty("content", new Gson().toJson(src.content));
            return o;
        }
    }

}