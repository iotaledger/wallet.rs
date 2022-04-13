export type RemainderValueStrategy = {
    ChangeAddress: {
        strategy: 'ChangeAddress';
        value: null
    },
    ReuseAddress: {
        strategy: 'ReuseAddress';
        value: null
    },
    AccountAddress: {
        strategy: 'AccountAddress';
        value: string;
    },
}

export interface NativeTokenOptions {
    // TODO: Change to camelCase
    // TOOD: Should this be just "address"?
    account_address: string;
    // TOOD: Change to proper type
    token_tag: any;
    // TOOD: Change to camelCase
    circulating_supply: number;
    // TOOD: Change to camelCase
    maximum_supply: number;
}

export interface NftOptions {
    address: string;
    // TOOD: Change to proper type
    immutable_metadata: any;
    // TOOD: Change to proper type
    metadata: any;
}

// TODO: Transfer & TransferOptions should probably be merged
export interface TransferOptions {
    // TODO: Change to camelCase
    remainder_value_strategy: RemainderValueStrategy;
    // TODO: Change to proper type
    tagged_data_payload: any;
    // TODO: Change to camelCase
    skip_sync: boolean;
    // TODO: Change to camelCase
    custom_inputs: string[];
}
