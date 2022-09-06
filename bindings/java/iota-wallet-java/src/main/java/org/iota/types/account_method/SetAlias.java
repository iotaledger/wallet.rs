package org.iota.types.account_method;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import org.iota.types.AddressesAndNftId;
import org.iota.types.JsonUtils;

public class SetAlias implements AccountMethod {

    private String alias;

    public SetAlias withAlias(String alias) {
        this.alias = alias;
        return this;
    }

    @Override
    public JsonElement toJson() {
        JsonObject o = new JsonObject();
        o.addProperty("alias", alias);

        return o;
    }
}