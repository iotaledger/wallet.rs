/** Wallet event types */
export type EventType =
    | '*'
    | 'ErrorThrown'
    | 'ConsolidationRequired'
    | 'LedgerAddressGeneration'
    | 'NewOutput'
    | 'SpentOutput'
    | 'TransactionInclusion'
    | 'TransactionProgress';

/** Wallet events */
export enum WalletEvent {
    ConsolidationRequired = 'ConsolidationRequired',
    LedgerAddressGeneration = 'LedgerAddressGeneration',
    NewOutput = 'NewOutput',
    SpentOutput = 'SpentOutput',
    TransactionInclusion = 'TransactionInclusion',
    TransactionProgress = 'TransactionProgress',
}
