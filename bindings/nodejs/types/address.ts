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

export interface AddressWithMicroAmount {
    address: string;
    amount: number;
    returnAddress: string;
    expiration: number;
}

export interface AddressNativeTokens {
    address: string;
    nativeTokens: string[];
    returnAddress: string;
    expiration: number;
}

export interface AddressNftId {
    address: string;
    nftId: string;
}
