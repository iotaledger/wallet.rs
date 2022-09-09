package org.iota.types.account_methods;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonPrimitive;
import org.iota.types.AccountAddress;
import org.iota.JsonUtils;
import org.iota.types.ReturnJson;
import org.iota.types.TaggedDataPayload;
import org.iota.types.ids.OutputId;

public class TransactionOptions implements ReturnJson {
    private RemainderValueStrategy remainderValueStrategy = new ReuseAddress();
    private TaggedDataPayload taggedDataPayload;
    private OutputId[] customInputs;
    private boolean allowBurning;
    private String note;

    public TransactionOptions withRemainderValueStrategy(RemainderValueStrategy remainderValueStrategy) {
        this.remainderValueStrategy = remainderValueStrategy;
        return this;
    }

    public TransactionOptions withTaggedDataPayload(TaggedDataPayload taggedDataPayload) {
        this.taggedDataPayload = taggedDataPayload;
        return this;
    }

    public TransactionOptions withCustomInputs(OutputId[] customInputs) {
        this.customInputs = customInputs;
        return this;
    }

    public TransactionOptions withAllowBurning(boolean allowBurning) {
        this.allowBurning = allowBurning;
        return this;
    }

    public TransactionOptions withNote(String note) {
        this.note = note;
        return this;
    }

    public interface RemainderValueStrategy extends ReturnJson {}

    public static class ReuseAddress implements  RemainderValueStrategy {
        @Override
        public JsonElement toJson() {
            return new JsonPrimitive("RemainderValueStrategy");
        }
    }
    public static class ChangeAddress implements  RemainderValueStrategy {
        @Override
        public JsonElement toJson() {
            return new JsonPrimitive("ChangeAddress");
        }
    }
    public static class CustomAddress implements  RemainderValueStrategy {
        private AccountAddress accountAddress;

        public CustomAddress withAccountAddress(AccountAddress accountAddress) {
            this.accountAddress = accountAddress;
            return this;
        }

        public CustomAddress(AccountAddress accountAddress) {
            this.accountAddress = accountAddress;
        }

        @Override
        public JsonElement toJson() {
            return accountAddress.toJson();
        }
    }

    @Override
    public JsonElement toJson() {
        JsonObject o = new JsonObject();
        o.add("remainderValueStrategy", remainderValueStrategy.toJson());
        o.add("taggedDataPayload", taggedDataPayload.toJson());
        o.add("customInputs", JsonUtils.toJson(customInputs));
        o.addProperty("allowBurning", allowBurning);
        o.addProperty("note", note);

        return o;
    }
}