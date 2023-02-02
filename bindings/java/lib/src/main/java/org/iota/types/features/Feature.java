// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.features;

import com.google.gson.*;
import com.google.gson.annotations.JsonAdapter;
import org.iota.types.addresses.Address;
import org.iota.types.addresses.AliasAddress;
import org.iota.types.addresses.Ed25519Address;
import org.iota.types.addresses.NftAddress;

import java.lang.reflect.Type;
@JsonAdapter(Feature.FeatureAdapter.class)
public abstract class Feature {

    public static class FeatureAdapter implements JsonDeserializer<Feature>, JsonSerializer<Feature> {

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

        public JsonElement serialize(Feature src, Type typeOfSrc, JsonSerializationContext context) {
            if (src instanceof SenderFeature) {
                return new Gson().toJsonTree(src, SenderFeature.class);
            } else if (src instanceof IssuerFeature) {
                return new Gson().toJsonTree(src, IssuerFeature.class);
            } else if (src instanceof MetadataFeature) {
                return new Gson().toJsonTree(src, MetadataFeature.class);
            } else if (src instanceof TagFeature) {
                return new Gson().toJsonTree(src, TagFeature.class);
            } else throw new JsonParseException("unknown class: " + src.getClass().getSimpleName());
        }

    }

}