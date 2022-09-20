package org.iota.types;

import com.google.gson.*;
import org.iota.types.ids.OutputId;
import org.iota.types.payload.TaggedDataPayload;

import java.lang.reflect.Type;

public class TransactionOptions {

    private RemainderValueStrategy remainderValueStrategy = new ReuseAddress();
    private TaggedDataPayload taggedDataPayload;
    private OutputId[] customInputs;
    private boolean allowBurning;
    private String note;

    public static abstract class RemainderValueStrategy implements JsonDeserializer<RemainderValueStrategy> {

        @Override
        public RemainderValueStrategy deserialize(final JsonElement json, final Type typeOfT, final JsonDeserializationContext context)
                throws JsonParseException {

            JsonObject jsonObject = json.getAsJsonObject();

            String type = jsonObject.get("strategy").getAsString();

            RemainderValueStrategy strategy;

            switch (type) {
                case "ReuseAddress": {
                    strategy = new Gson().fromJson(json, ReuseAddress.class);
                    break;
                }
                case "ChangeAddress": {
                    strategy = new Gson().fromJson(json, ChangeAddress.class);
                    break;
                }
                case "CustomAddress": {
                    strategy = new Gson().fromJson(json, CustomAddress.class);
                    break;
                }

                default: throw new JsonParseException("unknown type: " + type);
            }

            return strategy;
        }

    }

    public static class ReuseAddress extends RemainderValueStrategy {}
    public static class ChangeAddress extends RemainderValueStrategy {}
    public static class CustomAddress extends RemainderValueStrategy {
        private AccountAddress content;

        public CustomAddress(AccountAddress content) {
            this.content = content;
        }

        public AccountAddress getContent() {
            return content;
        }
    }

}