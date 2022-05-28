import type { IOutputMetadataResponse, ITransactionEssence, ITransactionPayload, OutputTypes } from '@iota/types';
import type { Segment } from './output';
import type { RemainderData } from './remainderData';

enum InclusionState {
    Pending = 'Pending',
    Confirmed = 'Confirmed',
    Conflicting = 'Conflicting',
}

// TODO replace with IPreparedTransactionData from iota.rs once exposed in @iota/types
export interface PreparedTransactionData {
    essence: ITransactionEssence;
    inputsData: InputsData[];
    remainder?: RemainderData 
}

interface InputsData {
    output: OutputTypes;
    outputMetadata: IOutputMetadataResponse;
    chain: Segment[];
    bech32Address: string;
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
