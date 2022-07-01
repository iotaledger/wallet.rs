import type { ITransactionPayload } from '@iota/types';

export enum InclusionState {
    Pending = 'Pending',
    Confirmed = 'Confirmed',
    Conflicting = 'Conflicting',
    UnknownPruned = 'UnknownPruned',
}

export interface Transaction {
    payload: ITransactionPayload;
    blockId?: string;
    inclusionState: InclusionState;
    timestamp: string;
    networkId: string;
    incoming: boolean;
}

export interface TransactionResult {
    transactionId: string;
    transaction: ITransactionPayload;
    blockId?: string;
}
