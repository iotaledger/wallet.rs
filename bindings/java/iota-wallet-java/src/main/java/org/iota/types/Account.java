package org.iota.types;

import com.google.gson.JsonObject;
import com.google.gson.JsonSyntaxException;

public class Account extends AbstractObject {
    public Account(JsonObject jsonObject) {
        super(jsonObject);
    }

    public Account(String jsonObject) throws JsonSyntaxException {
        super(jsonObject);
    }
}
