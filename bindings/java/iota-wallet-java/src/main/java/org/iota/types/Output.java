package org.iota.types;

import com.google.gson.JsonObject;
import com.google.gson.JsonSyntaxException;

public class Output extends AbstractObject {

    public Output(JsonObject jsonObject) {
        super(jsonObject);
    }

    public Output(String jsonObject) throws JsonSyntaxException {
        super(jsonObject);
    }
}