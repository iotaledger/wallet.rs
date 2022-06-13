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
    expiration?: Time;
    timelock?: Time;
}

interface StorageDeposit {
    returnStrategy?: ReturnStrategy
    useExcessIfLow?: boolean;
}

interface Time {
    milestoneIndex?: number;
    unixTime?: string;
}

enum ReturnStrategy {
    Return = 'Return',
    Gift = 'Gift',
}
