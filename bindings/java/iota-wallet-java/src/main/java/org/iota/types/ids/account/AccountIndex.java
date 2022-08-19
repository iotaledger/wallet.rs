package org.iota.types.ids.account;

import java.util.Objects;

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
}
