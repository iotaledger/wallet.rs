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
    address: string;
    // TOOD: Change to proper type
    immutableMetadata: any;
    // TOOD: Change to proper type
    metadata: any;
}

// TODO: Transfer & TransferOptions should probably be merged
export interface TransferOptions {
    remainderValueStrategy: RemainderValueStrategy;
    // TODO: Change to proper type
    taggedDataPayload: any;
    skipSync: boolean;
    customInputs: string[];
}
