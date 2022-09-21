package org.iota.types.addresses;

import com.google.gson.*;
import com.google.gson.annotations.JsonAdapter;
import org.iota.types.AbstractObject;
import org.iota.types.ids.AliasId;
import org.iota.types.ids.NftId;
import org.iota.types.unlock_conditions.*;

import java.lang.reflect.Type;

@JsonAdapter(Address.AddressAdapter.class)
public abstract class Address extends AbstractObject {

    class AddressAdapter implements JsonDeserializer<Address>, JsonSerializer<Address> {

        @Override
        public Address deserialize(final JsonElement json, final Type typeOfT, final JsonDeserializationContext context)
                throws JsonParseException {

            JsonObject jsonObject = json.getAsJsonObject();
            int addressType = jsonObject.get("type").getAsInt();

            Address address;

            switch (addressType) {
                case 0: {
                    address = new Ed25519Address(jsonObject.get("pubKeyHash").getAsString());
                    break;
                }
                case 8: {
                    address = new AliasAddress(new AliasId(jsonObject.get("aliasId").getAsString()));
                    break;
                }
                case 16: {
                    address = new NftAddress(new NftId(jsonObject.get("nftId").getAsString()));
                    break;
                }
                default: throw new JsonParseException("invalid address type: " + addressType);
            }

            return address;
        }

        public JsonElement serialize(Address src, Type typeOfSrc, JsonSerializationContext context) {
            if (src instanceof Ed25519Address) {
                return new Gson().toJsonTree(src, Ed25519Address.class);
            } else if (src instanceof AliasAddress) {
                return new Gson().toJsonTree(src, AliasAddress.class);
            } else if (src instanceof NftAddress) {
                return new Gson().toJsonTree(src, NftAddress.class);
            } else throw new JsonParseException("unknown class: " + src.getClass().getSimpleName());
        }

    }

}

