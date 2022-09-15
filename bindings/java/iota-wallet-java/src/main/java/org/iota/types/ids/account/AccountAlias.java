package org.iota.types.ids.account;

import com.google.gson.JsonElement;
import com.google.gson.JsonPrimitive;
import com.google.gson.JsonSerializationContext;
import com.google.gson.JsonSerializer;
import com.google.gson.annotations.JsonAdapter;

import java.lang.reflect.Type;
import java.util.Objects;

@JsonAdapter(AccountAlias.AccountAliasAdapter.class)
public class AccountAlias extends AccountIdentifier {
    private String accountAlias;

    public AccountAlias(String accountAlias) {
        this.accountAlias = accountAlias;
    }

    @Override
    public boolean equals(Object o) {
        if (this == o) return true;
        if (o == null || getClass() != o.getClass()) return false;
        AccountAlias other = (AccountAlias) o;
        return Objects.equals(this.accountAlias, other.accountAlias);
    }

    @Override
    public int hashCode() {
        return Objects.hash(accountAlias);
    }

    public String getAccountAlias() {
        return accountAlias;
    }

    public static class AccountAliasAdapter implements JsonSerializer<AccountAlias> {
        @Override
        public JsonElement serialize(AccountAlias src, Type typeOfSrc, JsonSerializationContext context) {
            return new JsonPrimitive(src.getAccountAlias());
        }
    }
}
