import type { ITransactionEssence, ITransactionPayload } from '@iota/types';
import type { RemainderValueStrategy } from './transfer';

enum InclusionState {
    Pending = 'Pending',
    Confirmed = 'Confirmed',
    Conflicting = 'Conflicting',
}

export interface PreparedTransaction {
    essence: ITransactionEssence;
    // TODO replace with IInputSigningData from iota.rs once exposed in @iota/types
    inputsData: any[];
    remainder?: RemainderValueStrategy 
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
    blockId?: string;
}
