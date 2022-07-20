# Interface: PreparedTransactionData

Prepared transaction data, useful for offline signing.

## Table of contents

### Properties

- [essence](PreparedTransactionData.md#essence)
- [inputsData](PreparedTransactionData.md#inputsdata)
- [remainder](PreparedTransactionData.md#remainder)

## Properties

### essence

• **essence**: `ITransactionEssence`

Transaction essence

___

### inputsData

• **inputsData**: [`InputSigningData`](InputSigningData.md)[]

Required address information for signing

___

### remainder

• `Optional` **remainder**: [`RemainderData`](RemainderData.md)

Optional remainder output information
