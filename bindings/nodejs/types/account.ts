import type { Address } from './address';
import type { OutputData } from './output';
import type { Transaction } from './transaction';

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
    addresses?: string[]
    addressStartIndex?: number;
    automaticOutputConsolidation?: boolean;
    forceSyncing?: boolean;
    syncPendingTransactions?: boolean;
    syncAliasesAndNfts?: boolean;
    tryCollectOutputs?: boolean;
    outputConsolidationThreshold?: number;
}

export interface AccountMeta {
    index: number;
    coinType: CoinType;
    alias: string;
    publicAddresses: Address[];
    internalAddresses: Address[];
    addressesWithBalance: Address[];
    outputs: OutputsMap;
    lockedOutputs: Set<string>;
    unspentOutputs: OutputsMap;
    transactions: TransactionsMap;
    pendingTransactions: Set<string>;
    accountOptions: {
        outputConsolidationThreshold: number;
        automaticOutputConsolidation: boolean;
    };
}

export type OutputsMap = {
    [outputId: string]: OutputData;
};

export type TransactionsMap = {
    [transactionId: string]: Transaction;
};

export enum CoinType {
    IOTA = 4218,
    Shimmer = 4219,
}

export interface CreateAccountPayload {
    alias?: string
    coinType?: CoinType
}
