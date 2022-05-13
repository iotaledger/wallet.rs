export type EventType =
    | '*'
    | 'ErrorThrown'
    | 'BalanceChange'
    | 'NewTransaction'
    | 'ConfirmationStateChange'
    | 'Reattachment'
    | 'Broadcast'
    | 'TransferProgress';

export enum WalletEvent {
    BalanceChange = 'BalanceChange',
    TransactionInclusion = 'TransactionInclusion',
    TransferProgress = 'TransferProgress',
    ConsolidationRequired = 'ConsolidationRequired',
    LedgerAddressGeneration = 'LedgerAddressGeneration',
}
