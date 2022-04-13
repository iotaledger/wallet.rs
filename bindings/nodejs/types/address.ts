export interface Address {
    address: string;
    keyIndex: number;
    internal: boolean;
    used: boolean;
}

export interface AddressWithAmount {
    address: string;
    amount: number;
}

export interface AddressMicroAmount {
    address: string;
    amount: number;
    // TODO: Change to camelCase
    return_address: string;
    expiration: number
}

export interface AddressNativeTokens {
    address: string;
    // TODO: Change to camelCase
    native_tokens: string[];
    // TODO: Change to camelCase
    return_address: string;
    expiration: number;
}

export interface AddressNftId {
    address: string;
    // TODO: Change to camelCase
    nft_id: string;
}
