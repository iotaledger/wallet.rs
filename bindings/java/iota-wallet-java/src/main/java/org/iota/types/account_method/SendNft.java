package org.iota.types.account_method;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import org.iota.types.AddressesAndNftId;
import org.iota.types.JsonUtils;

public class SendNft implements AccountMethod {

    private AddressesAndNftId[] addressesAndNftIds;
    private TransactionOptions options;

    @Override
    public JsonElement toJson() {
        JsonObject o = new JsonObject();
        o.add("addressesAndNftIds", JsonUtils.toJson(addressesAndNftIds));
        o.add("options", options.toJson());

        return o;
    }
}