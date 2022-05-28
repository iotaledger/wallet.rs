import type { AddressTypes, IOutputResponse } from '@iota/types';

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

export interface OutputData {
    outputId: string;
    outputResponse: IOutputResponse;
    output: Output;
    amount: string;
    isSpent: boolean;
    address: {
        type: AddressTypes;
        data: string;
    };
    networkId: string;
    remainder: boolean;
    chain?: Segment[];
}

// TODO: this should be the same as bee_block::output::Output
export interface Output {
    type: OutputType;
    // TODO: specify the return type 
    data: any;
}

export interface Segment {
    hardened: boolean;
    bs: number[];
}
