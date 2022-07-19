# Interface: InputSigningData

Data for transaction inputs for signing and ordering of unlock blocks

## Table of contents

### Properties

- [output](InputSigningData.md#output)
- [outputMetaData](InputSigningData.md#outputmetadata)
- [chain](InputSigningData.md#chain)
- [bech32Address](InputSigningData.md#bech32address)

## Properties

### output

• **output**: `OutputTypes`

The output

___

### outputMetaData

• **outputMetaData**: `IOutputMetadataResponse`

The output metadata

___

### chain

• `Optional` **chain**: [`Segment`](Segment.md)[]

The chain derived from seed, only for ed25519 addresses

___

### bech32Address

• **bech32Address**: `string`

The bech32 encoded address, required because of alias outputs where we have multiple possible unlock
conditions, because we otherwise don't know which one we need
