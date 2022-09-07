package org.iota.types;

import com.google.gson.JsonObject;
import com.google.gson.JsonSyntaxException;

public class NftOptions extends AbstractObject {

    public NftOptions(JsonObject jsonObject) {
        super(jsonObject);
    }

    public NftOptions(String jsonObject) throws JsonSyntaxException {
        super(jsonObject);
    }
}