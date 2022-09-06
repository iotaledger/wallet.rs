package org.iota.types;

import com.google.gson.JsonObject;
import com.google.gson.JsonSyntaxException;

public class SyncOptions extends AbstractObject {
    public SyncOptions(JsonObject jsonObject) {
        super(jsonObject);
    }

    public SyncOptions(String jsonObject) throws JsonSyntaxException {
        super(jsonObject);
    }
}
