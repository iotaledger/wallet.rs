package org.iota.types.account_methods;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import org.iota.types.SyncOptions;

public class SyncAccount implements AccountMethod {

    private SyncOptions options;

    public SyncAccount withOptions(SyncOptions options) {
        this.options = options;
        return this;
    }

    @Override
    public JsonElement toJson() {
        JsonObject o = new JsonObject();
        o.add("options", options.toJson());

        return o;
    }
}