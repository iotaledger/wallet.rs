package org.iota.types;

import com.google.gson.JsonObject;
import com.google.gson.JsonSyntaxException;

public class AccountAddress extends AbstractObject {
    public AccountAddress(JsonObject jsonObject) {
        super(jsonObject);
    }

    public AccountAddress(String jsonObject) throws JsonSyntaxException {
        super(jsonObject);
    }
}
