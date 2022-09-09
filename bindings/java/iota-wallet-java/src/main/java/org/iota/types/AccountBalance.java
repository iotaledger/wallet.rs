package org.iota.types;

import com.google.gson.JsonObject;
import com.google.gson.JsonSyntaxException;

public class AccountBalance extends AbstractObject {
    public AccountBalance(JsonObject jsonObject) {
        super(jsonObject);
    }

    public AccountBalance(String jsonObject) throws JsonSyntaxException {
        super(jsonObject);
    }
}