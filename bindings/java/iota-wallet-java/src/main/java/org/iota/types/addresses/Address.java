package org.iota.types.addresses;

import com.google.gson.*;
import com.google.gson.annotations.JsonAdapter;
import org.iota.types.ids.AliasId;
import org.iota.types.ids.NftId;

import java.lang.reflect.Type;

@JsonAdapter(Address.AddressAdapter.class)
public abstract class Address {

    class AddressAdapter implements JsonDeserializer<Address> {

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

    }

}

