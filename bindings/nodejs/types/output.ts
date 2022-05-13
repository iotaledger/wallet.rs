import type { IOutputResponse, OutputTypes, AddressTypes } from '@iota/types';

export enum OutputsToCollect {
    None = 'None',
    MicroTransactions = 'MicroTransactions',
    NativeTokens = 'NativeTokens',
    Nfts = 'Nfts',
    All = 'All',
}

enum Output {
    Treasury = 'Treasury',
    Basic = 'Basic',
    Alias = 'Alias',
    Foundry = 'Foundry',
    Nft = 'Nft',
}

export interface OutputResponse {
    messageId: string;
    transactionId: string;
    outputIndex: number;
    isSpent: boolean;
    milestoneIndexSpent?: number;
    milestoneTimestampSpent?: number;
    transactionIdSpent?: string;
    milestoneIndexBooked: number;
    milestoneTimestampBooked: number;
    ledgerIndex: number;
    output: Output;
}

export interface OutputData {
    outputId: string;
    outputResponse: OutputResponse;
    output: Output;
    amount: number;
    isSpent: boolean;
    address: AddressTypes;
    networkId: number;
    remainder: boolean;
    chain: Segment[];
}

export interface Segment {
    hardened: boolean;
    bs: number[];
}
