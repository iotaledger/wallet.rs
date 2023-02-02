// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.unlock_conditions;

import com.google.gson.*;
import com.google.gson.annotations.JsonAdapter;
import org.iota.types.AbstractObject;
import org.iota.types.outputs.*;

import java.lang.reflect.Type;
@JsonAdapter(UnlockCondition.UnlockConditionAdapter.class)
public abstract class UnlockCondition extends AbstractObject {

    public static class UnlockConditionAdapter implements JsonDeserializer<UnlockCondition>, JsonSerializer<UnlockCondition> {

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
        public JsonElement serialize(UnlockCondition src, Type typeOfSrc, JsonSerializationContext context) {
            if (src instanceof AddressUnlockCondition) {
                return new Gson().toJsonTree(src, AddressUnlockCondition.class);
            } else if (src instanceof StorageDepositReturnUnlockCondition) {
                return new Gson().toJsonTree(src, StorageDepositReturnUnlockCondition.class);
            } else if (src instanceof TimelockUnlockCondition) {
                return new Gson().toJsonTree(src, TimelockUnlockCondition.class);
            } else if (src instanceof ExpirationUnlockCondition) {
                return new Gson().toJsonTree(src, ExpirationUnlockCondition.class);
            } else if (src instanceof StateControllerAddressUnlockCondition) {
                return new Gson().toJsonTree(src, StateControllerAddressUnlockCondition.class);
            } else if (src instanceof GovernorAddressUnlockCondition) {
                return new Gson().toJsonTree(src, GovernorAddressUnlockCondition.class);
            } else if (src instanceof ImmutableAliasAddressUnlockCondition) {
                return new Gson().toJsonTree(src, ImmutableAliasAddressUnlockCondition.class);
            } else throw new JsonParseException("unknown class: " + src.getClass().getSimpleName());
        }

    }

}