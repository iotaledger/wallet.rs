// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.unlock_conditions;

import org.iota.types.addresses.Address;
public class ImmutableAliasAddressUnlockCondition extends UnlockCondition {

        private int type = 6;
        private Address address;

        public int getType() {
                return type;
        }

        public Address getAddress() {
                return address;
        }

        public ImmutableAliasAddressUnlockCondition withAddress(Address address) {
                this.address = address;
                return this;
        }
}