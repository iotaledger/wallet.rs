
import type { AccountSyncOptions, CreateAccountPayload } from './account';
import type { AddressWithAmount, AddressWithMicroAmount, AddressNativeTokens, AddressNftId } from './address';
import type { ClientOptions } from './network';
import type { OutputsToCollect, OutputData } from './output';
import type { NativeTokenOptions, TransferOptions, NftOptions } from './transfer';

type __GetAccountsMessagePayload__ = {
    cmd: 'getAccounts'
}

type __GetAccountMessagePayload__ = {
    cmd: 'getAccount'
    payload: string
}

type __CreateAccountMessagePayload__ = {
    cmd: 'createAccount'
    payload: CreateAccountPayload
}

type __SetStrongholdPasswordPayload__ = {
    cmd: 'setStrongholdPassword'
    payload: string;
}

type __StoreMnemonicPayload__ = {
    cmd: 'storeMnemonic'
    payload: {
        signerType: {
            type: 'Stronghold'
        },
        mnemonic: string
    };
}

type __BackupPayload__ = {
    cmd: 'backup'
    payload: {
        destination: string
        password: string
    };
}

type __ImportAccountsPayload__ = {
    cmd: 'restoreBackup'
    payload: {
        backupPath: string
        password: string
    };
}

type __SyncAccountPayload__ = {
    cmd: 'callAccountMethod'
    payload: {
        accountId: number;
        method: {
            name: 'syncAccount',
            data?: AccountSyncOptions
        }
    };
}

type __GetNodeInfoPayload__ = {
    cmd: 'callAccountMethod'
    payload: {
        accountId: number;
        method: {
            name: 'getNodeInfo',
            data: string[]
        }
    };
}

type __GenerateAddressesPayload__ = {
    cmd: 'callAccountMethod'
    payload: {
        accountId: number;
        method: {
            name: 'generateAddresses',
            data: {
                amount: number
            }
        }
    };
}

type __LatestAddressPayload__ = {
    cmd: 'callAccountMethod'
    payload: {
        accountId: number;
        method: {
            name: 'getLatestAddress'
        }
    };
}

type __BalancePayload__ = {
    cmd: 'callAccountMethod'
    payload: {
        accountId: number;
        method: {
            name: 'getBalance'
        }
    };
}

type __SetClientOptionsPayload__ = {
    cmd: 'setClientOptions'
    payload: ClientOptions;
}

type __SetCollectOutputsPayload__ = {
    cmd: 'callAccountMethod'
    payload: {
        accountId: number;
        method: {
            name: 'collectOutputs',
            data: {
                outputIdsToCollect: string[]
            }
        }
    };
}

type __GetOutputsWithAdditionalUnlockConditionsPayload__ = {
    cmd: 'callAccountMethod'
    payload: {
        accountId: number;
        method: {
            name: 'getOutputsWithAdditionalUnlockConditions',
            data: {
                outputsToCollect: OutputsToCollect
            }
        }
    };
}

type __ListAddressesPayload__ = {
    cmd: 'callAccountMethod'
    payload: {
        accountId: number;
        method: {
            name: 'listAddresses'
        }
    };
}

type __ListAddressesWithBalancePayload__ = {
    cmd: 'callAccountMethod'
    payload: {
        accountId: number;
        method: {
            name: 'listAddressesWithBalance'
        }
    };
}

type __ListOutputsPayload__ = {
    cmd: 'callAccountMethod'
    payload: {
        accountId: number;
        method: {
            name: 'listOutputs'
        }
    };
}

type __ListPendingTransactionsPayload__ = {
    cmd: 'callAccountMethod'
    payload: {
        accountId: number;
        method: {
            name: 'listPendingTransactions'
        }
    };
}

type __ListTransactionsPayload__ = {
    cmd: 'callAccountMethod'
    payload: {
        accountId: number;
        method: {
            name: 'listTransactions'
        }
    };
}

type __ListUnspentOutputsPayload__ = {
    cmd: 'callAccountMethod'
    payload: {
        accountId: number;
        method: {
            name: 'listUnspentOutputs'
        }
    };
}

type __MintNativeTokenPayload__ = {
    cmd: 'callAccountMethod'
    payload: {
        accountId: number;
        method: {
            name: 'mintNativeToken',
            data: {
                nativeTokenOptions: NativeTokenOptions;
                options: TransferOptions
            }
        }
    };
}

type __MintNftsPayload__ = {
    cmd: 'callAccountMethod'
    payload: {
        accountId: number;
        method: {
            name: 'mintNfts',
            data: {
                nftsOptions: NftOptions;
                options: TransferOptions
            }
        }
    };
}

type __SendAmountPayload__ = {
    cmd: 'callAccountMethod'
    payload: {
        accountId: number;
        method: {
            name: 'sendAmount',
            data: {
                addressesWithAmount: AddressWithAmount[];
                options: TransferOptions
            }
        }
    };
}

type __SendMicroTransactionPayload__ = {
    cmd: 'callAccountMethod'
    payload: {
        accountId: number;
        method: {
            name: 'sendMicroTransaction',
            data: {
                addressesWithMicroAmount: AddressWithMicroAmount[];
                options: TransferOptions
            }
        }
    };
}

type __SendNativeTokensPayload__ = {
    cmd: 'callAccountMethod'
    payload: {
        accountId: number;
        method: {
            name: 'sendNativeTokens',
            data: {
                addressesNativeTokens: AddressNativeTokens[];
                options: TransferOptions
            }
        }
    };
}

type __SendNftPayload__ = {
    cmd: 'callAccountMethod'
    payload: {
        accountId: number;
        method: {
            name: 'sendNft',
            data: {
                addressesNftIds: AddressNftId[];
                options: TransferOptions
            }
        }
    };
}

type __SendTransferPayload__ = {
    cmd: 'callAccountMethod'
    payload: {
        accountId: number;
        method: {
            name: 'sendTransfer',
            data: {
                outputs: OutputData[];
                options: TransferOptions
            }
        }
    };
}

type __TryCollectOutputsPayload__ = {
    cmd: 'callAccountMethod'
    payload: {
        accountId: number;
        method: {
            name: 'tryCollectOutputs',
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
