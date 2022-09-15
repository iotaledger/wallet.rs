package org.iota.api;

import com.google.gson.Gson;
import com.google.gson.GsonBuilder;

public class GsonSingleton {

    private static Gson GSON = new GsonBuilder().create();

    public static Gson getInstance() {
        return GSON;
    }
    
}