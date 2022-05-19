import type { ITransactionPayload } from '@iota/types';

enum InclusionState {
    Pending = 'Pending',
    Confirmed = 'Confirmed',
    Conflicting = 'Conflicting',
}

export interface Transaction {
    payload: ITransactionPayload;
    messageId?: string;
    inclusionState: InclusionState;
    timestamp: number;
    networkId: number;
    incoming: boolean;
}

export interface TransactionReceipt {
    transactionId: string;
    messageId?: string
}
