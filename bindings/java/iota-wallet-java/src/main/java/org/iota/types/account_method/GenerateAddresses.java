package org.iota.types.account_method;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import org.iota.types.ReturnJson;

public class GenerateAddresses implements AccountMethod {

    private int amount;
    private AddressGenerationOptions addressGenerationOptions;

    public GenerateAddresses withAmount(int amount) {
        this.amount = amount;
        return this;
    }

    public GenerateAddresses withAddressGenerationOptions(AddressGenerationOptions addressGenerationOptions) {
        this.addressGenerationOptions = addressGenerationOptions;
        return this;
    }

    public static class AddressGenerationOptions implements ReturnJson {
        private boolean internal;
        private GenerateAddressMetadata metadata;

        @Override
        public JsonElement toJson() {
            JsonObject o = new JsonObject();
            o.addProperty("internal", internal);
            o.add("metadata", metadata.toJson());

            return o;
        }

        public static class GenerateAddressMetadata implements ReturnJson {
            private boolean syncing;

            public GenerateAddressMetadata withSyncing(boolean syncing) {
                this.syncing = syncing;
                return this;
            }

            @Override
            public JsonElement toJson() {
                JsonObject o = new JsonObject();
                o.addProperty("syncing", syncing);

                return o;
            }
        }
    }

    @Override
    public JsonElement toJson() {
        JsonObject o = new JsonObject();
        o.addProperty("amount", amount);
        o.add("addressGenerationOptions", addressGenerationOptions.toJson());

        return o;
    }
}