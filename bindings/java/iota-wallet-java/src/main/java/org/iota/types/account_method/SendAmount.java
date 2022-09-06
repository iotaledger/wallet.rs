package org.iota.types.account_method;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import org.iota.types.AddressWithAmount;
import org.iota.types.JsonUtils;
import org.iota.types.SyncOptions;

public class SendAmount implements AccountMethod {

    private AddressWithAmount[] addressesWithAmount;
    private TransactionOptions options;

    public SendAmount withAddressesWithAmount(AddressWithAmount[] addressesWithAmount) {
        this.addressesWithAmount = addressesWithAmount;
        return this;
    }

    public SendAmount withOptions(TransactionOptions options) {
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