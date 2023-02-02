// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types;
public class FilterOptions extends AbstractObject {

    /// Filter all outputs where the booked milestone index is below the specified timestamp
    private Integer lowerBoundBookedTimestamp;
    /// Filter all outputs where the booked milestone index is above the specified timestamp
    private Integer upperBoundBookedTimestamp;
    /// Filter all outputs for the provided types (Basic = 3, Alias = 4, Foundry = 5, NFT = 6)
    private Integer[] outputTypes;

    public FilterOptions withLowerBoundBookedTimestamp(Integer lowerBoundBookedTimestamp) {
        this.lowerBoundBookedTimestamp = lowerBoundBookedTimestamp;
        return this;
    }

    public FilterOptions withUpperBoundBookedTimestamp(Integer upperBoundBookedTimestamp) {
        this.upperBoundBookedTimestamp = upperBoundBookedTimestamp;
        return this;
    }

    public FilterOptions withOutputTypes(Integer[] outputTypes) {
        this.outputTypes = outputTypes;
        return this;
    }

    public Integer getLowerBoundBookedTimestamp() {
        return lowerBoundBookedTimestamp;
    }

    public Integer getUpperBoundBookedTimestamp() {
        return upperBoundBookedTimestamp;
    }

    public Integer[] getOutputTypes() {
        return outputTypes;
    }
}
