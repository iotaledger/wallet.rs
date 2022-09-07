package org.iota.types;

import com.google.gson.JsonObject;
import com.google.gson.JsonSyntaxException;

public class NativeTokenOptions extends AbstractObject {

    public NativeTokenOptions(JsonObject jsonObject) {
        super(jsonObject);
    }

    public NativeTokenOptions(String jsonObject) throws JsonSyntaxException {
        super(jsonObject);
    }
}