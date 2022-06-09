import type { INativeToken } from '@iota/types';
import type { Address, AddressWithUnspentOutputs } from './address';
import type { OutputData, OutputId } from './output';
import type { Transaction } from './transaction';

/**
 * Account identifier
 * Could be the account index (number) or account alias (string)
 */
export type AccountId = number | string;

export interface AccountBalance {
    total: string;
    available: string;
    requiredStorageDeposit: string;
    nativeTokens: INativeToken[];
    nfts: string[];
    aliases: string[];
    foundries: string[];
    potentiallyLockedOutputs: potentiallyLockedOutputs;
}

/**
 * Outputs with multiple unlock conditions and if they can currently be spent or not. If there is a
 * TimelockUnlockCondition or ExpirationUnlockCondition this can change at any time
 */
export type potentiallyLockedOutputs = [OutputId, boolean];

export interface AccountSyncOptions {
    addresses?: string[];
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
    addressesWithUnspentOutputs: AddressWithUnspentOutputs[];
    outputs: Map<string, OutputData>;
    lockedOutputs: Set<string>;
    unspentOutputs: Map<string, OutputData>;
    transactions: Map<string, Transaction>;
    pendingTransactions: Set<string>;
}

export enum CoinType {
    IOTA = 4218,
    Shimmer = 4219,
}

export interface CreateAccountPayload {
    alias?: string;
    coinType?: CoinType;
}
