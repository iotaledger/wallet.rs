package org.iota.types.account_methods;

import org.iota.types.FilterOptions;

/// Returns all unspent outputs of the account
public class ListUnspentOutputs implements AccountMethod {

    private FilterOptions filterOptions;

    public ListUnspentOutputs withFilterOptions(FilterOptions filterOptions) {
        this.filterOptions = filterOptions;
        return this;
    }

}