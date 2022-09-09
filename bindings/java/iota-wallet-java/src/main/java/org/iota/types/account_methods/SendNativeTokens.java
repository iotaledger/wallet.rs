package org.iota.types.account_methods;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import org.iota.types.AddressNativeTokens;
import org.iota.JsonUtils;

public class SendNativeTokens implements AccountMethod {

    private AddressNativeTokens[] addressesNativeTokens;
    private TransactionOptions options;

    public SendNativeTokens withAddressesNativeTokens(AddressNativeTokens[] addressesNativeTokens) {
        this.addressesNativeTokens = addressesNativeTokens;
        return this;
    }

    public SendNativeTokens withOptions(TransactionOptions options) {
        this.options = options;
        return this;
    }

    @Override
    public JsonElement toJson() {
        JsonObject o = new JsonObject();
        o.add("addressesWithMicroAmount", JsonUtils.toJson(addressesNativeTokens));
        o.add("options", options.toJson());

        return o;
    }
}