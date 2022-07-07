import type { Network } from './network';

export enum AddressType {
    Ed25519 = 'Ed25519',
    Alias = 'Alias',
    Nft = 'Nft',
}

export interface Address {
    address: string;
    keyIndex: number;
    internal: boolean;
    used: boolean;
}

export interface AddressWithAmount {
    address: string;
    amount: string;
}

export interface AddressWithUnspentOutputs {
    address: string;
    keyIndex: number;
    internal: boolean;
    outputIds: string[];
}

export interface AddressWithMicroAmount {
    address: string;
    amount: string;
    returnAddress?: string;
    expiration?: number;
}

export interface AddressNativeTokens {
    address: string;
    nativeTokens: string[];
    returnAddress?: string;
    expiration?: number;
}

export interface AddressNftId {
    address: string;
    nftId: string;
}

export interface AddressGenerationOptions {
    internal: boolean;
    metadata: GenerateAddressMetadata;
}

export interface GenerateAddressMetadata {
    syncing: boolean;
    network: Network;
}
