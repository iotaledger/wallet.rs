import type { ITaggedDataPayload } from '@iota/types';

export interface TransactionOptions {
    remainderValueStrategy?: RemainderValueStrategy;
    taggedDataPayload?: ITaggedDataPayload;
    customInputs?: string[];
}

export type RemainderValueStrategy =
    | ChangeAddress
    | ReuseAddress
    | CustomAddress;

export type ChangeAddress = {
    strategy: 'ChangeAddress';
    value: null;
};

export type ReuseAddress = {
    strategy: 'ReuseAddress';
    value: null;
};

export type CustomAddress = {
    strategy: 'CustomAddress';
    value: string;
};

export interface NativeTokenOptions {
    accountAddress?: string;
    /** Hex encoded number */
    circulatingSupply: string;
    /** Hex encoded number */
    maximumSupply: string;
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
