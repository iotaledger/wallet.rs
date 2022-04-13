
export enum OutputsToCollect {
    None = 'None',
    MicroTransactions = 'MicroTransactions',
    NativeTokens = 'NativeTokens',
    Nfts = 'Nfts',
    All = 'All',
}

export interface OutputResponse {
    messageId: string;
    transactionId: string;
    outputIndex: number;
    isSpent: boolean;
    milestoneIndexBooked: number;
    milestoneTimestampBooked: number;
    ledgerIndex: number;
    // TODO: Replace with proper type
    output: any;
}

export interface OutputData {
    outputId: string;
    outputResponse: OutputResponse;
    // TODO: Replace with proper type
    output: any;
    amount: number;
    isSpent: boolean;
    address: {
        // TODO: Also add other types
        type: 'Ed25519',
        data: string;
    },
    networkId: number;
    remainder: boolean;
    // TODO: Replace with proper type
    chain: any;
}
