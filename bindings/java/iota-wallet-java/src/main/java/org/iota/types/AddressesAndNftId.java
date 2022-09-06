package org.iota.types;

import com.google.gson.JsonObject;
import com.google.gson.JsonSyntaxException;

public class AddressesAndNftId extends AbstractObject {
    public AddressesAndNftId(JsonObject jsonObject) {
        super(jsonObject);
    }

    public AddressesAndNftId(String jsonObject) throws JsonSyntaxException {
        super(jsonObject);
    }
}
