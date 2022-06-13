import type {
    AddressTypes,
    OutputTypes,
    IOutputMetadataResponse,
    INativeToken,
} from '@iota/types';

export enum OutputsToCollect {
    None = 'None',
    MicroTransactions = 'MicroTransactions',
    NativeTokens = 'NativeTokens',
    Nfts = 'Nfts',
    All = 'All',
}

/** An output with metadata */
export interface OutputData {
    /** The identifier of an Output */
    outputId: string;
    /** The metadata of the output */
    metadata: IOutputMetadataResponse;
    /** The actual Output */
    output: OutputTypes;
    /** The output amount */
    amount: string;
    /** If an output is spent */
    isSpent: boolean;
    /** Associated account address */
    address: AddressTypes;
    /** Network ID */
    networkId: string;
    /** Remainder */
    remainder: boolean;
    /** Bip32 path */
    chain?: Segment[];
}

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

export interface Segment {
    hardened: boolean;
    bs: number[];
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
