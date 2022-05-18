import type { ITaggedDataPayload } from '@iota/types';

export type RemainderValueStrategy = {
    ChangeAddress: {
        strategy: 'ChangeAddress';
        value: null;
    };
    ReuseAddress: {
        strategy: 'ReuseAddress';
        value: null;
    };
    AccountAddress: {
        strategy: 'AccountAddress';
        value: string;
    };
};

export interface NativeTokenOptions {
    // TOOD: Should this be just "address"?
    accountAddress: string;
    // TOOD: Change to proper type
    tokenTag: any;
    circulatingSupply: number;
    maximumSupply: number;
}

export interface NftOptions {
    /** Bech32 encoded address to which the Nft will be minted. Default will use the
     * first address of the account
     */
    address?: string;
    immutableMetadata?: number[];
    metadata?: number[];
}

// TODO: Transfer & TransferOptions should probably be merged
export interface TransferOptions {
    remainderValueStrategy: RemainderValueStrategy;
    taggedDataPayload?: ITaggedDataPayload;
    skipSync: boolean;
    customInputs?: string[];
}
