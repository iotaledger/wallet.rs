// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types;

public abstract class AbstractTuple {

    private Object[] values;

    public AbstractTuple(Object... values) {
        this.values = values;
    }

    protected Object get(int index) {
        return values[index];
    }

}
