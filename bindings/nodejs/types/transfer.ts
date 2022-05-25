import type { ITaggedDataPayload } from '@iota/types';

export type RemainderValueStrategy = ChangeAddress | ReuseAddress | AccountAddress

type ChangeAddress = {
    strategy: 'ChangeAddress';
    value: null;
};

type ReuseAddress = {
    strategy: 'ReuseAddress';
    value: null;
};

type AccountAddress = {
    strategy: 'CustomAddress';
    value: string;
};

export interface NativeTokenOptions {
    accountAddress?: string;
    circulatingSupply: number;
    maximumSupply: number;
    foundryMetadata?: number[];
}

export interface NftOptions {
    /** Bech32 encoded address to which the Nft will be minted. Default will use the
     * first address of the account
     */
    address?: string;
    immutableMetadata?: number[];
    metadata?: number[];
}

export interface TransferOptions {
    remainderValueStrategy?: RemainderValueStrategy;
    taggedDataPayload?: ITaggedDataPayload;
    skipSync?: boolean;
    customInputs?: string[];
}
