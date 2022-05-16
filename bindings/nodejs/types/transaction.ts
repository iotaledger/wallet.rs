enum InclusionState {
    Pending = 'Pending',
    Confirmed = 'Confirmed',
    Conflicting = 'Conflicting',
}

// TODO: properly type this
interface TransactionPayload {
    essence: any;
    unlockBlocks: any
}

export interface Transaction {
    payload: TransactionPayload;
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
