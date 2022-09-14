package org.iota.types.account_methods;

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

    public static class AddressGenerationOptions {
        private boolean internal;
        private GenerateAddressMetadata metadata;

        public static class GenerateAddressMetadata {
            private boolean syncing;

            public GenerateAddressMetadata withSyncing(boolean syncing) {
                this.syncing = syncing;
                return this;
            }

        }
    }

}