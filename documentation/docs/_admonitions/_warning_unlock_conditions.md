:::warning Unlock Conditions

Outputs may have multiple [UnlockConditions](https://github.com/iotaledger/tips/blob/main/tips/TIP-0018/tip-0018.md#unlock-conditions) which may require returning some or all of the amount, which could expire if not claimed in time, or which may not be unlockable for a very long time.
To get only outputs with the `AddressUnlockCondition` alone that do not require additional ownership checks, synchronize with `syncOnlyMostBasicOutputs: true`. When synchronizing also other outputs, the unlock conditions must be carefully checked before crediting users any balance.

An example how to check if an output has only the address unlock condition, where the address belongs to the account, can be found [here](../how_tos/outputs_and_transactions/06_check_unlock_conditions.mdx).

:::