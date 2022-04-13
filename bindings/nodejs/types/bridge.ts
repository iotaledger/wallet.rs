
import type { AccountSyncOptions, CreateAccountPayload } from './account';
import type { AddressWithAmount, AddressMicroAmount, AddressNativeTokens, AddressNftId } from './address';
import type { ClientOptions } from './network';
import type { OutputsToCollect, OutputData } from './output';
import type { NativeTokenOptions, TransferOptions, NftOptions } from './transfer';

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
type __TryCollectOutputsPayload__ = {
    cmd: 'CallAccountMethod'
    payload: {
        account_id: number;
        method: {
            name: 'TryCollectOutputs',
            data: {
                outputs_to_collect: OutputsToCollect;
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
