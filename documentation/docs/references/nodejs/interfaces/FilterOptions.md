# Interface: FilterOptions

Options to filter outputs

## Table of contents

### Properties

- [lowerBoundBookedTimestamp](FilterOptions.md#lowerboundbookedtimestamp)
- [upperBoundBookedTimestamp](FilterOptions.md#upperboundbookedtimestamp)
- [outputTypes](FilterOptions.md#outputtypes)

## Properties

### lowerBoundBookedTimestamp

• `Optional` **lowerBoundBookedTimestamp**: `number`

Filter all outputs where the booked milestone index is below the specified timestamp

___

### upperBoundBookedTimestamp

• `Optional` **upperBoundBookedTimestamp**: `number`

Filter all outputs where the booked milestone index is above the specified timestamp

___

### outputTypes

• `Optional` **outputTypes**: `Uint8Array`

Filter all outputs for the provided types (Basic = 3, Alias = 4, Foundry = 5, NFT = 6)
