import type { AddressTypes } from '@iota/types';

export enum OutputsToCollect {
    None = 'None',
    MicroTransactions = 'MicroTransactions',
    NativeTokens = 'NativeTokens',
    Nfts = 'Nfts',
    All = 'All',
}

enum OutputType {
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
    output: OutputDTO;
}

interface OutputDTO {
    type: number;
    amount: string;
    // TODO: specify unlockConditions type
    unlockConditions: any[]
}

export interface OutputData {
    outputId: string;
    outputResponse: OutputResponse;
    output: Output;
    amount: number;
    isSpent: boolean;
    address: {
        type: AddressTypes,
        data: string;
    },
    networkId: number;
    remainder: boolean;
    chain: Segment[];
}

export interface Output {
    type: OutputType;
    // TODO: specify the return type
    data: any
}

export interface Segment {
    hardened: boolean;
    bs: number[];
}
