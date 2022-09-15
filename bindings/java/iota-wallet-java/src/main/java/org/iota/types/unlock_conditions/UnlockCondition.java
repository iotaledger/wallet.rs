package org.iota.types.unlock_conditions;

import com.google.gson.*;
import com.google.gson.annotations.JsonAdapter;

import java.lang.reflect.Type;

@JsonAdapter(UnlockCondition.UnlockConditionAdapter.class)
public abstract class UnlockCondition {

    public static class UnlockConditionAdapter implements JsonDeserializer<UnlockCondition> {

        @Override
        public UnlockCondition deserialize(final JsonElement json, final Type typeOfT, final JsonDeserializationContext context)
                throws JsonParseException {

            int type = json.getAsJsonObject().get("type").getAsInt();

            UnlockCondition unlockCondition;

            switch (type) {
                case 0: {
                    unlockCondition = new Gson().fromJson(json, AddressUnlockCondition.class);
                    break;
                }
                case 1: {
                    unlockCondition = new Gson().fromJson(json, StorageDepositReturnUnlockCondition.class);
                    break;
                }
                case 2: {
                    unlockCondition = new Gson().fromJson(json, TimelockUnlockCondition.class);
                    break;
                }
                case 3: {
                    unlockCondition = new Gson().fromJson(json, ExpirationUnlockCondition.class);
                    break;
                }
                case 4: {
                    unlockCondition = new Gson().fromJson(json, StateControllerAddressUnlockCondition.class);
                    break;
                }
                case 5: {
                    unlockCondition = new Gson().fromJson(json, GovernorAddressUnlockCondition.class);
                    break;
                }
                case 6: {
                    unlockCondition = new Gson().fromJson(json, ImmutableAliasAddressUnlockCondition.class);
                    break;
                }


                default: throw new JsonParseException("unknown type: " + type);
            }

            return unlockCondition;
        }

    }

}