// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

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