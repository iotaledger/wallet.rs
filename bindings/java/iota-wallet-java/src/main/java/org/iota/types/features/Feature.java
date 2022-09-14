package org.iota.types.features;

import com.google.gson.*;
import org.iota.types.secret.SecretManager;

import java.lang.reflect.Type;

public abstract class Feature implements JsonDeserializer<Feature> {

    @Override
    public Feature deserialize(final JsonElement json, final Type typeOfT, final JsonDeserializationContext context)
            throws JsonParseException {

        JsonObject jsonObject = json.getAsJsonObject();

        int type = jsonObject.get("type").getAsInt();

        Feature feature;

        switch (type) {
            case 0: {
                feature = new Gson().fromJson(json, SenderFeature.class);
                break;
            }
            case 1: {
                feature = new Gson().fromJson(json, IssuerFeature.class);
                break;
            }
            case 2: {
                feature = new Gson().fromJson(json, MetadataFeature.class);
                break;
            }
            case 3: {
                feature = new Gson().fromJson(json, TagFeature.class);
                break;
            }

            default: throw new JsonParseException("unknown type: " + type);
        }

        return feature;
    }

}