package org.iota.types.account_method;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import org.iota.types.AddressWithAmount;
import org.iota.types.AddressWithMicroAmount;
import org.iota.types.JsonUtils;

public class SendMicroTransaction implements AccountMethod {

    private AddressWithMicroAmount[] addressesWithMicroAmount;
    private TransactionOptions options;

    public SendMicroTransaction withAddressesWithAmount(AddressWithMicroAmount[] addressesWithMicroAmount) {
        this.addressesWithMicroAmount = addressesWithMicroAmount;
        return this;
    }

    public SendMicroTransaction withOptions(TransactionOptions options) {
        this.options = options;
        return this;
    }

    @Override
    public JsonElement toJson() {
        JsonObject o = new JsonObject();
        o.add("addressesWithMicroAmount", JsonUtils.toJson(addressesWithMicroAmount));
        o.add("options", options.toJson());

        return o;
    }
}