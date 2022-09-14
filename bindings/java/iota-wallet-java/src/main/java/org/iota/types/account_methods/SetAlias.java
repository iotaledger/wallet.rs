package org.iota.types.account_methods;

public class SetAlias implements AccountMethod {

    private String alias;

    public SetAlias withAlias(String alias) {
        this.alias = alias;
        return this;
    }

}