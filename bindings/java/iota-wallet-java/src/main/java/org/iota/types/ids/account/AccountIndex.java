package org.iota.types.ids.account;

import com.google.gson.JsonElement;
import com.google.gson.JsonPrimitive;
import com.google.gson.JsonSerializationContext;
import com.google.gson.JsonSerializer;
import com.google.gson.annotations.JsonAdapter;

import java.lang.reflect.Type;
import java.util.Objects;

@JsonAdapter(AccountIndex.AccountIndexAdapter.class)
public class AccountIndex extends AccountIdentifier {
    private Integer accountIndex;

    public AccountIndex(int accountIndex) {
        this.accountIndex = accountIndex;
    }

    @Override
    public boolean equals(Object o) {
        if (this == o) return true;
        if (o == null || getClass() != o.getClass()) return false;
        AccountIndex other = (AccountIndex) o;
        return Objects.equals(this.accountIndex, other.accountIndex);
    }

    @Override
    public int hashCode() {
        return Objects.hash(accountIndex);
    }

    public Integer getAccountIndex() {
        return accountIndex;
    }

    public static class AccountIndexAdapter implements JsonSerializer<AccountIndex> {
        @Override
        public JsonElement serialize(AccountIndex src, Type typeOfSrc, JsonSerializationContext context) {
            return new JsonPrimitive(src.getAccountIndex());
        }
    }
}
