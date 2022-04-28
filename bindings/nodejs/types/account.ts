import type { Address } from './address';

/**
 * Account identifier
 * Could be the account index (number) or account alias (string)
 */
export type AccountId = number | string;

export interface AccountBalance {
    total: number;
    available: number;
    incoming: number;
    outgoing: number;
}

export interface AccountSyncOptions {
    addressIndex?: number;
    gapLimit?: number;
}

export interface AccountMeta {
    index: number;
    // TODO: Should this be an enum?
    coinType: number;
    alias: string;
    publicAddresses: Address[];
    internalAddresses: Address[];
    addressesWithBalance: Address[];
    // TODO: Define type for outputs
    outputs: any;
    lockedOutputs: any;
    unspentOutputs: any;
    // TODO: Define type for outputs
    transactions: any;
    pendingTransactions: any;
    accountOptions: {
        outputConsolidationThreshold: number;
        automaticOutputConsolidation: boolean;
    };
}

export enum CoinType {
    IOTA = 4218,
    Shimmer = 4219,
}

export interface CreateAccountPayload {
    alias: string;
    coinType?: CoinType;
}
