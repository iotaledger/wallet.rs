export type EventType =
    | '*'
    | 'ErrorThrown'
    | 'BalanceChange'
    | 'NewTransaction'
    | 'ConfirmationStateChange'
    | 'Reattachment'
    | 'Broadcast'
    | 'TransactionProgress';

export enum WalletEvent {
    BalanceChange = 'BalanceChange',
    TransactionInclusion = 'TransactionInclusion',
    TransactionProgress = 'TransactionProgress',
    ConsolidationRequired = 'ConsolidationRequired',
    LedgerAddressGeneration = 'LedgerAddressGeneration',
}
