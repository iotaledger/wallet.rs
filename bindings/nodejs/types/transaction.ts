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
    transactionId: string;
    networkId: string;
    incoming: boolean;
}

export interface MintTokenTransaction {
    tokenId: string;
    transaction: Transaction;
}