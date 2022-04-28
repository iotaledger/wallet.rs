import type { IOutputResponse, OutputTypes, AddressTypes } from '@iota/types';

export enum OutputsToCollect {
    None = 'None',
    MicroTransactions = 'MicroTransactions',
    NativeTokens = 'NativeTokens',
    Nfts = 'Nfts',
    All = 'All',
}

export interface OutputData {
    outputId: string;
    outputResponse: IOutputResponse;
    output: OutputTypes;
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
