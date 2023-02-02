// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.account_methods;

import org.iota.types.FilterOptions;

/// Returns all outputs of the account
public class Outputs implements AccountMethod {

    /// Options to filter outputs
    private FilterOptions filterOptions;

    public Outputs withFilterOptions(FilterOptions filterOptions) {
        this.filterOptions = filterOptions;
        return this;
    }
}