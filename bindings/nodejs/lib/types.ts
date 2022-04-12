
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

export interface Transfer {
    address: string;
    amount: number;
    remainder_value_strategy: RemainderValueStrategy;
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
type __SendPayload__ = {
    cmd: 'SendTransfer'
    payload: {
        account_id: number;
        transfer: Transfer
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
