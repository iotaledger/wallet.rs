import type {
    AddressTypes,
    OutputTypes,
    IOutputMetadataResponse,
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

export interface Segment {
    hardened: boolean;
    bs: number[];
}
