package org.iota.types.account_methods;

/// Set the alias of the account.
public class SetAlias implements AccountMethod {

    private String alias;

    public SetAlias withAlias(String alias) {
        this.alias = alias;
        return this;
    }

}