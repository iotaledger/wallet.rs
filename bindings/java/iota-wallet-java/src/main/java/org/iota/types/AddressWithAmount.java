package org.iota.types;

import com.google.gson.JsonObject;
import com.google.gson.JsonSyntaxException;

public class AddressWithAmount extends AbstractObject {
    public AddressWithAmount(JsonObject jsonObject) {
        super(jsonObject);
    }

    public AddressWithAmount(String jsonObject) throws JsonSyntaxException {
        super(jsonObject);
    }
}
