
export enum Network {
    Mainnet,
    Testnet,
}

export enum OutputsToCollect {
    None = 'None',
    MicroTransactions = 'MicroTransactions',
    NativeTokens = 'NativeTokens',
    Nfts = 'Nfts',
    All = 'All',
}

// TODO: Update Transaction interface
export interface Transaction {
    // TODO: Change to camelCase
    message_id: string;
    // TODO: Add other inclusion states
    inclusion_state: 'Confirmed';
    timestamp: number;
    // TODO: Change to camelCase
    network_id: number;
    incoming: boolean;
}

export interface OutputResponse {
    messageId: string;
    transactionId: string;
    outputIndex: number;
    isSpent: boolean;
    milestoneIndexBooked: number;
    milestoneTimestampBooked: number;
    ledgerIndex: number;
    // TODO: Replace with proper type
    output: any;
}

export interface OutputData {
    outputId: string;
    outputResponse: OutputResponse;
    // TODO: Replace with proper type
    output: any;
    amount: number;
    isSpent: boolean;
    address: {
        // TODO: Also add other types
        type: 'Ed25519',
        data: string;
    },
    networkId: number;
    remainder: boolean;
    // TODO: Replace with proper type
    chain: any;
}

export type RemainderValueStrategy = {
    ChangeAddress: {
        strategy: 'ChangeAddress';
        value: null
    },
    ReuseAddress: {
        strategy: 'ReuseAddress';
        value: null
    },
    AccountAddress: {
        strategy: 'AccountAddress';
        value: string;
    },
}

export interface NativeTokenOptions {
    // TODO: Change to camelCase
    // TOOD: Should this be just "address"?
    account_address: string;
    // TOOD: Change to proper type
    token_tag: any;
    // TOOD: Change to camelCase
    circulating_supply: number;
    // TOOD: Change to camelCase
    maximum_supply: number;
}

export interface NftOptions {
    address: string;
    // TOOD: Change to proper type
    immutable_metadata: any;
    // TOOD: Change to proper type
    metadata: any;
}

// TODO: Transfer & TransferOptions should probably be merged
export interface TransferOptions {
    // TODO: Change to camelCase
    remainder_value_strategy: RemainderValueStrategy;
    // TODO: Change to proper type
    tagged_data_payload: any;
    // TODO: Change to camelCase
    skip_sync: boolean;
    // TODO: Change to camelCase
    custom_inputs: string[];
}

export interface AccountBalance {
    total: number;
    available: number;
    incoming: number;
    outgoing: number;
}

export interface NodeInfo {
    name: string;
    version: string;
    isHealthy: boolean;
    networkId: string;
    bech32HRP: string;
    minPoWScore: number;
    messagesPerSecond: number;
    referencedMessagesPerSecond: number;
    referencedRate: number;
    latestMilestoneTimestamp: number;
    latestMilestoneIndex: number;
    confirmedMilestoneIndex: number;
    pruningIndex: number;
    features: string[];
}

export interface AccountSyncOptions {
    addressIndex?: number;
    gapLimit?: number;
}

export type Address = {
    address: string;
    keyIndex: number;
    internal: boolean;
    used: boolean;
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

export type EventType =
    | '*'
    | 'ErrorThrown'
    | 'BalanceChange'
    | 'NewTransaction'
    | 'ConfirmationStateChange'
    | 'Reattachment'
    | 'Broadcast'
    | 'TransferProgress';

export type Auth = {
    jwt?: string;
    username?: string;
    password?: string;
}

export interface AddressWithAmount {
    address: string;
    amount: number;
}

export interface AddressMicroAmount {
    address: string;
    amount: number;
    // TODO: Change to camelCase
    return_address: string;
    expiration: number
}

export interface AddressNativeTokens {
    address: string;
    // TODO: Change to camelCase
    native_tokens: string[];
    // TODO: Change to camelCase
    return_address: string;
    expiration: number;
}

export interface AddressNftId {
    address: string;
    // TODO: Change to camelCase
    nft_id: string;
}

export interface MqttBrokerOptions {
    automaticDisconnect?: boolean;
    // timeout in seconds
    timeout?: number;
    useWs?: boolean;
    port?: number;
    maxReconnectionAttempts?: number;
}

export type Node = {
    url: string;
    auth?: Auth;
    disabled?: boolean;
}

export interface ClientOptions {
    primaryNode?: string | Node;
    primaryPoWNode?: string | Node;
    node?: string | Node;
    nodes?: Array<string | Node>;
    network?: string;
    mqttBrokerOptions?: MqttBrokerOptions;
    quorumSize?: number;
    quorumThreshold?: number;
    localPow?: boolean;
}

export interface AccountManagerOptions {
    storagePath?: string;
    clientOptions?: ClientOptions;
    signer?: string;
}

export interface CreateAccountPayload {
    alias: string
}

type __GetAccountsMessagePayload__ = {
    cmd: 'GetAccounts'
}
type __GetAccountMessagePayload__ = {
    cmd: 'GetAccount'
    payload: string
}
type __CreateAccountMessagePayload__ = {
    cmd: 'CreateAccount'
    payload: CreateAccountPayload
}
type __SetStrongholdPasswordPayload__ = {
    cmd: 'SetStrongholdPassword'
    payload: string;
}
type __StoreMnemonicPayload__ = {
    cmd: 'StoreMnemonic'
    payload: {
        signerType: {
            type: 'Stronghold'
        },
        mnemonic: string
    };
}
type __BackupPayload__ = {
    cmd: 'Backup'
    payload: {
        destination: string
        password: string
    };
}
type __ImportAccountsPayload__ = {
    cmd: 'RestoreBackup'
    payload: {
        backupPath: string
        password: string
    };
}
type __SyncAccountPayload__ = {
    cmd: 'CallAccountMethod'
    payload: {
        account_id: number;
        method: {
            name: 'SyncAccount',
            data?: AccountSyncOptions
        }
    };
}
type __GetNodeInfoPayload__ = {
    cmd: 'CallAccountMethod'
    payload: {
        account_id: number;
        method: {
            name: 'GetNodeInfo',
            data: string[]
        }
    };
}
type __GenerateAddressesPayload__ = {
    cmd: 'CallAccountMethod'
    payload: {
        account_id: number;
        method: {
            name: 'GenerateAddresses',
            data: {
                amount: number
            }
        }
    };
}
type __LatestAddressPayload__ = {
    cmd: 'CallAccountMethod'
    payload: {
        account_id: number;
        method: {
            name: 'GetLatestAddress'
        }
    };
}
type __BalancePayload__ = {
    cmd: 'CallAccountMethod'
    payload: {
        account_id: number;
        method: {
            name: 'GetBalance'
        }
    };
}
type __SetClientOptionsPayload__ = {
    cmd: 'SetClientOptions'
    payload: ClientOptions;
}
type __SetCollectOutputsPayload__ = {
    cmd: 'CallAccountMethod'
    payload: {
        account_id: number;
        method: {
            name: 'CollectOutputs',
            data: {
                output_ids_to_collect: string[]
            }
        }
    };
}
type __GetOutputsWithAdditionalUnlockConditionsPayload__ = {
    cmd: 'CallAccountMethod'
    payload: {
        account_id: number;
        method: {
            name: 'GetOutputsWithAdditionalUnlockConditions',
            data: {
                outputs_to_collect: OutputsToCollect
            }
        }
    };
}
type __ListAddressesPayload__ = {
    cmd: 'CallAccountMethod'
    payload: {
        account_id: number;
        method: {
            name: 'ListAddresses'
        }
    };
}
type __ListAddressesWithBalancePayload__ = {
    cmd: 'CallAccountMethod'
    payload: {
        account_id: number;
        method: {
            name: 'ListAddressesWithBalance'
        }
    };
}
type __ListOutputsPayload__ = {
    cmd: 'CallAccountMethod'
    payload: {
        account_id: number;
        method: {
            name: 'ListOutputs'
        }
    };
}
type __ListPendingTransactionsPayload__ = {
    cmd: 'CallAccountMethod'
    payload: {
        account_id: number;
        method: {
            name: 'ListPendingTransactions'
        }
    };
}
type __ListTransactionsPayload__ = {
    cmd: 'CallAccountMethod'
    payload: {
        account_id: number;
        method: {
            name: 'ListTransactions'
        }
    };
}
type __ListUnspentOutputsPayload__ = {
    cmd: 'CallAccountMethod'
    payload: {
        account_id: number;
        method: {
            name: 'ListUnspentOutputs'
        }
    };
}
type __MintNativeTokenPayload__ = {
    cmd: 'CallAccountMethod'
    payload: {
        account_id: number;
        method: {
            name: 'MintNativeToken',
            data: {
                native_token_options: NativeTokenOptions;
                options: TransferOptions
            }
        }
    };
}
type __MintNftsPayload__ = {
    cmd: 'CallAccountMethod'
    payload: {
        account_id: number;
        method: {
            name: 'MintNfts',
            data: {
                nfts_options: NftOptions;
                options: TransferOptions
            }
        }
    };
}
type __SendAmountPayload__ = {
    cmd: 'CallAccountMethod'
    payload: {
        account_id: number;
        method: {
            name: 'SendAmount',
            data: {
                addresses_with_amount: AddressWithAmount[];
                options: TransferOptions
            }
        }
    };
}
type __SendMicroTransactionPayload__ = {
    cmd: 'CallAccountMethod'
    payload: {
        account_id: number;
        method: {
            name: 'SendMicroTransaction',
            data: {
                addresses_with_micro_amount: AddressMicroAmount[];
                options: TransferOptions
            }
        }
    };
}
type __SendNativeTokensPayload__ = {
    cmd: 'CallAccountMethod'
    payload: {
        account_id: number;
        method: {
            name: 'SendNativeTokens',
            data: {
                addresses_native_tokens: AddressNativeTokens[];
                options: TransferOptions
            }
        }
    };
}
type __SendNftPayload__ = {
    cmd: 'CallAccountMethod'
    payload: {
        account_id: number;
        method: {
            name: 'SendNft',
            data: {
                addresses_nft_ids: AddressNftId[];
                options: TransferOptions
            }
        }
    };
}
type __SendTransferPayload__ = {
    cmd: 'CallAccountMethod'
    payload: {
        account_id: number;
        method: {
            name: 'SendTransfer',
            data: {
                outputs: OutputData[];
                options: TransferOptions
            }
        }
    };
}

export type __SendMessagePayload__ =
    | __GetAccountsMessagePayload__
    | __GetAccountMessagePayload__
    | __CreateAccountMessagePayload__
    | __SetStrongholdPasswordPayload__
    | __StoreMnemonicPayload__
    | __BackupPayload__
    | __ImportAccountsPayload__
    | __SyncAccountPayload__
    | __GetNodeInfoPayload__
    | __GenerateAddressesPayload__
    | __LatestAddressPayload__
    | __BalancePayload__
    | __SendPayload__
    | __SetClientOptionsPayload__
    | __SetCollectOutputsPayload__
    | __GetOutputsWithAdditionalUnlockConditionsPayload__
    | __ListAddressesPayload__
    | __ListAddressesWithBalancePayload__
    | __ListOutputsPayload__
    | __ListPendingTransactionsPayload__
    | __ListTransactionsPayload__
    | __ListUnspentOutputsPayload__
    | __MintNativeTokenPayload__
    | __MintNftsPayload__
    | __SendAmountPayload__
    | __SendMicroTransactionPayload__
    | __SendNativeTokensPayload__
    | __SendNftPayload__
    | __SendTransferPayload__;
