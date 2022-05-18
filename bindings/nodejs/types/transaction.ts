import type { ITransactionPayload } from '@iota/types';

export interface Transaction {
    payload: ITransactionPayload;
    messageId?: string;
    inclusionState: 'Pending' | 'Confirmed' | 'Conflicting';
    timestamp: number;
    networkId: number;
    incoming: boolean;
}
