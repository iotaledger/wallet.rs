import type { INativeToken } from '@iota/types';

export interface OutputOptions {
    recipientAddress: string;
    amount: string;
    assets?: Assets;
    features?: Features;
    unlocks?: Unlocks;
    storageDeposit?: StorageDeposit
}

interface Assets {
    nativeTokens?: INativeToken[];
    nftId?: string;
}

interface Features {
    tag?: string;
    metadata?: string;
}

interface Unlocks {
    expirationUnixTime?: number;
    timelockUnixTime?: number;
}

interface StorageDeposit {
    returnStrategy?: ReturnStrategy
    useExcessIfLow?: boolean;
}

enum ReturnStrategy {
    Return = 'Return',
    Gift = 'Gift',
}
