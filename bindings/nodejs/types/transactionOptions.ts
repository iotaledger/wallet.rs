import type { ITaggedDataPayload } from '@iota/types';

export interface TransactionOptions {
    remainderValueStrategy?: RemainderValueStrategy;
    taggedDataPayload?: ITaggedDataPayload;
    skipSync?: boolean;
    customInputs?: string[];
}

export type RemainderValueStrategy =
    | ChangeAddress
    | ReuseAddress
    | CustomAddress;

type ChangeAddress = {
    strategy: 'ChangeAddress';
    value: null;
};

type ReuseAddress = {
    strategy: 'ReuseAddress';
    value: null;
};

type CustomAddress = {
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
