package org.iota;

import com.google.gson.JsonArray;

public class JsonUtils<T> {

    public static JsonArray toJson(byte[] array) {
        if (array != null) {
            JsonArray a = new JsonArray();
            for (byte b : array) {
                a.add(b & 0xFF);
            }
            return a;
        } else {
            return null;
        }
    }

}
