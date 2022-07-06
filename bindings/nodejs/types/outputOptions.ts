import type { INativeToken } from '@iota/types';

export interface OutputOptions {
    recipientAddress: string;
    amount: string;
    assets?: Assets;
    features?: Features;
    unlocks?: Unlocks;
    storageDeposit?: StorageDeposit;
}

export interface Assets {
    nativeTokens?: INativeToken[];
    nftId?: string;
}

export interface Features {
    tag?: string;
    metadata?: string;
}

export interface Unlocks {
    expirationUnixTime?: number;
    timelockUnixTime?: number;
}

export interface StorageDeposit {
    returnStrategy?: ReturnStrategy;
    useExcessIfLow?: boolean;
}

export enum ReturnStrategy {
    Return = 'Return',
    Gift = 'Gift',
}
