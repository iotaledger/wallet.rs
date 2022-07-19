# Interface: OutputData

An output with metadata

## Table of contents

### Properties

- [outputId](OutputData.md#outputid)
- [metadata](OutputData.md#metadata)
- [output](OutputData.md#output)
- [isSpent](OutputData.md#isspent)
- [address](OutputData.md#address)
- [networkId](OutputData.md#networkid)
- [remainder](OutputData.md#remainder)
- [chain](OutputData.md#chain)

## Properties

### outputId

• **outputId**: `string`

The identifier of an Output

___

### metadata

• **metadata**: `IOutputMetadataResponse`

The metadata of the output

___

### output

• **output**: `OutputTypes`

The actual Output

___

### isSpent

• **isSpent**: `boolean`

If an output is spent

___

### address

• **address**: `AddressTypes`

Associated account address

___

### networkId

• **networkId**: `string`

Network ID

___

### remainder

• **remainder**: `boolean`

Remainder

___

### chain

• `Optional` **chain**: `Segment`[]

Bip32 path
