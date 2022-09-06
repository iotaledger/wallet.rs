package org.iota.types;

import com.google.gson.JsonObject;
import com.google.gson.JsonSyntaxException;

public class AddressNativeTokens extends AbstractObject {
    public AddressNativeTokens(JsonObject jsonObject) {
        super(jsonObject);
    }

    public AddressNativeTokens(String jsonObject) throws JsonSyntaxException {
        super(jsonObject);
    }
}
