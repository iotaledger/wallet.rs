package org.iota.types;

import com.google.gson.JsonObject;
import com.google.gson.JsonSyntaxException;

public class AddressWithMicroAmount extends AbstractObject {
    public AddressWithMicroAmount(JsonObject jsonObject) {
        super(jsonObject);
    }

    public AddressWithMicroAmount(String jsonObject) throws JsonSyntaxException {
        super(jsonObject);
    }
}
