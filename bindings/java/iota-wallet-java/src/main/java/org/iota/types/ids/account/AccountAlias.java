package org.iota.types.ids.account;

import java.util.Objects;

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
}
