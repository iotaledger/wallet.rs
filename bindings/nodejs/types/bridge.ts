
import type { AccountId, AccountSyncOptions, CreateAccountPayload } from './account';
import type { AddressWithAmount, AddressWithMicroAmount, AddressNativeTokens, AddressNftId } from './address';
import type { ClientOptions } from './network';
import type { OutputsToCollect, OutputData } from './output';
import type { NativeTokenOptions, TransferOptions, NftOptions } from './transfer';

type __GetAccountsMessagePayload__ = {
    cmd: 'GetAccounts'
}

type __GetAccountMessagePayload__ = {
    cmd: 'GetAccount'
    payload: AccountId
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
        accountId: AccountId;
        method: {
            name: 'SyncAccount',
            data?: AccountSyncOptions
        }
    };
}

type __GetNodeInfoPayload__ = {
    cmd: 'CallAccountMethod'
    payload: {
        accountId: AccountId;
        method: {
            name: 'GetNodeInfo',
            data: string[]
        }
    };
}

type __GenerateAddressesPayload__ = {
    cmd: 'CallAccountMethod'
    payload: {
        accountId: AccountId;
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
        accountId: AccountId;
        method: {
            name: 'GetLatestAddress'
        }
    };
}

type __BalancePayload__ = {
    cmd: 'CallAccountMethod'
    payload: {
        accountId: AccountId;
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
        accountId: AccountId;
        method: {
            name: 'CollectOutputs',
            data: {
                outputIdsToCollect: string[]
            }
        }
    };
}

type __GetOutputsWithAdditionalUnlockConditionsPayload__ = {
    cmd: 'CallAccountMethod'
    payload: {
        accountId: AccountId;
        method: {
            name: 'GetOutputsWithAdditionalUnlockConditions',
            data: {
                outputsToCollect: OutputsToCollect
            }
        }
    };
}

type __ListAddressesPayload__ = {
    cmd: 'CallAccountMethod'
    payload: {
        accountId: AccountId;
        method: {
            name: 'ListAddresses'
        }
    };
}

type __ListAddressesWithBalancePayload__ = {
    cmd: 'CallAccountMethod'
    payload: {
        accountId: AccountId;
        method: {
            name: 'ListAddressesWithBalance'
        }
    };
}

type __ListOutputsPayload__ = {
    cmd: 'CallAccountMethod'
    payload: {
        accountId: AccountId;
        method: {
            name: 'ListOutputs'
        }
    };
}

type __ListPendingTransactionsPayload__ = {
    cmd: 'CallAccountMethod'
    payload: {
        accountId: AccountId;
        method: {
            name: 'ListPendingTransactions'
        }
    };
}

type __ListTransactionsPayload__ = {
    cmd: 'CallAccountMethod'
    payload: {
        accountId: AccountId;
        method: {
            name: 'ListTransactions'
        }
    };
}

type __ListUnspentOutputsPayload__ = {
    cmd: 'CallAccountMethod'
    payload: {
        accountId: AccountId;
        method: {
            name: 'ListUnspentOutputs'
        }
    };
}

type __MintNativeTokenPayload__ = {
    cmd: 'CallAccountMethod'
    payload: {
        accountId: AccountId;
        method: {
            name: 'MintNativeToken',
            data: {
                nativeTokenOptions: NativeTokenOptions;
                options: TransferOptions
            }
        }
    };
}

type __MintNftsPayload__ = {
    cmd: 'CallAccountMethod'
    payload: {
        accountId: AccountId;
        method: {
            name: 'MintNfts',
            data: {
                nftsOptions: NftOptions;
                options: TransferOptions
            }
        }
    };
}

type __SendAmountPayload__ = {
    cmd: 'CallAccountMethod'
    payload: {
        accountId: AccountId;
        method: {
            name: 'SendAmount',
            data: {
                addressesWithAmount: AddressWithAmount[];
                options: TransferOptions
            }
        }
    };
}

type __SendMicroTransactionPayload__ = {
    cmd: 'CallAccountMethod'
    payload: {
        accountId: AccountId;
        method: {
            name: 'SendMicroTransaction',
            data: {
                addressesWithMicroAmount: AddressWithMicroAmount[];
                options: TransferOptions
            }
        }
    };
}

type __SendNativeTokensPayload__ = {
    cmd: 'CallAccountMethod'
    payload: {
        accountId: AccountId;
        method: {
            name: 'SendNativeTokens',
            data: {
                addressesNativeTokens: AddressNativeTokens[];
                options: TransferOptions
            }
        }
    };
}

type __SendNftPayload__ = {
    cmd: 'CallAccountMethod'
    payload: {
        accountId: AccountId;
        method: {
            name: 'SendNft',
            data: {
                addressesNftIds: AddressNftId[];
                options: TransferOptions
            }
        }
    };
}

type __SendTransferPayload__ = {
    cmd: 'CallAccountMethod'
    payload: {
        accountId: AccountId;
        method: {
            name: 'SendTransfer',
            data: {
                outputs: OutputData[];
                options: TransferOptions
            }
        }
    };
}

type __TryCollectOutputsPayload__ = {
    cmd: 'CallAccountMethod'
    payload: {
        accountId: AccountId;
        method: {
            name: 'TryCollectOutputs',
            data: {
                outputsToCollect: OutputsToCollect;
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
    | __SendTransferPayload__
    | __TryCollectOutputsPayload__;
