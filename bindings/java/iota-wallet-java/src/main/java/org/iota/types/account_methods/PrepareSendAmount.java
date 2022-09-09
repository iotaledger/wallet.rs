package org.iota.types.account_methods;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import org.iota.types.AddressWithAmount;
import org.iota.JsonUtils;

public class PrepareSendAmount implements AccountMethod {

    private AddressWithAmount[] addressesWithAmount;
    private TransactionOptions options;

    public PrepareSendAmount withAddressesWithAmount(AddressWithAmount[] addressesWithAmount) {
        this.addressesWithAmount = addressesWithAmount;
        return this;
    }

    public PrepareSendAmount withOptions(TransactionOptions options) {
        this.options = options;
        return this;
    }

    @Override
    public JsonElement toJson() {
        JsonObject o = new JsonObject();
        o.add("addressesWithAmount", JsonUtils.toJson(addressesWithAmount));
        o.add("options", options.toJson());

        return o;
    }
}