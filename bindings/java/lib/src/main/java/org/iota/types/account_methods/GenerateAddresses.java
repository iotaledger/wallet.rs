// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.account_methods;

public class GenerateAddresses implements AccountMethod {

    private int amount;
    private AddressGenerationOptions options;

    public GenerateAddresses withAmount(int amount) {
        this.amount = amount;
        return this;
    }

    public GenerateAddresses withAddressGenerationOptions(AddressGenerationOptions options) {
        this.options = options;
        return this;
    }

    public static class AddressGenerationOptions {
        private boolean internal;
        private GenerateAddressOptions options;

        public AddressGenerationOptions withInternal(boolean internal) {
            this.internal = internal;
            return this;
        }

        public AddressGenerationOptions withOptions(GenerateAddressOptions options) {
            this.options = options;
            return this;
        }

        public static class GenerateAddressOptions {
            private boolean ledgerNanoPrompt;

            public GenerateAddressOptions withLedgerNanoPrompt(boolean ledgerNanoPrompt) {
                this.ledgerNanoPrompt = ledgerNanoPrompt;
                return this;
            }

        }
    }

}