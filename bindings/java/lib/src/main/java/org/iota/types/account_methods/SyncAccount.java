// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.account_methods;

import org.iota.types.SyncOptions;

/// Sync the account by fetching new information from the nodes. Will also retry pending transactions
/// if necessary.
public class SyncAccount implements AccountMethod {

    /// Sync options
    private SyncOptions options;

    public SyncAccount withOptions(SyncOptions options) {
        this.options = options;
        return this;
    }
}