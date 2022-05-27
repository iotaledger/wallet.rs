import type { ITransactionPayload } from '@iota/types';

enum InclusionState {
    Pending = 'Pending',
    Confirmed = 'Confirmed',
    Conflicting = 'Conflicting',
}

export interface Transaction {
    payload: ITransactionPayload;
    blockId?: string;
    inclusionState: InclusionState;
    timestamp: string;
    networkId: string;
    incoming: boolean;
}

export interface TransferResult {
    transactionId: string;
    blockId?: string;
}
