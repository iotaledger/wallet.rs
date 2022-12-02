// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.account_methods;

import org.iota.types.TransactionOptions;

/// Create an alias output.
public class CreateAliasOutput implements AccountMethod {

    private AliasOutputOptions aliasOutputOptions;
    private TransactionOptions options;


    public CreateAliasOutput withAliasOutputOptions(AliasOutputOptions aliasOutputOptions) {
        this.aliasOutputOptions = aliasOutputOptions;
        return this;
    }

    public CreateAliasOutput withOptions(TransactionOptions options) {
        this.options = options;
        return this;
    }

    public static class AliasOutputOptions {
        /// Bech32 encoded address which will control the alias. Default will use the first
        /// address of the account
        private String address;
        /// Immutable alias metadata, hex encoded bytes
        private String immutableMetadata;
        /// Alias metadata, hex encoded bytes
        private String metadata;
        /// Alias state metadata
        private String stateMetadata;

        public AliasOutputOptions withAddress(String address) {
            this.address = address;
            return this;
        }

        public AliasOutputOptions withImmutableMetadata(String immutableMetadata) {
            this.immutableMetadata = immutableMetadata;
            return this;
        }

        public AliasOutputOptions withMetadata(String metadata) {
            this.metadata = metadata;
            return this;
        }

        public AliasOutputOptions withStateMetadata(String stateMetadata) {
            this.stateMetadata = stateMetadata;
            return this;
        }
    }
}