
import type { Address } from './address';

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

// TODO: Change snakecase to camelCase
export interface AccountMeta {
    index: number;
    // TODO: Should this be an enum?
    coin_type: number;
    alias: string;
    public_addresses: Address[];
    internal_addresses: Address[];
    addresses_with_balance: Address[];
    // TODO: Define type for outputs
    outputs: any;
    locked_outputs: any;
    unspent_outputs: any;
    // TODO: Define type for outputs
    transactions: any;
    pending_transactions: any;
    account_options: {
        output_consolidation_threshold: number;
        automatic_output_consolidation: boolean;
    }
}

export interface CreateAccountPayload {
    alias: string
}
