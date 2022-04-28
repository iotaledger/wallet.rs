import type { AccountId, AccountSyncOptions } from '../account';
import type {
    AddressWithAmount,
    AddressWithMicroAmount,
    AddressNativeTokens,
    AddressNftId,
} from '../address';
import type { ClientOptions } from '../network';
import type { OutputsToCollect, OutputData } from '../output';
import type {
    NativeTokenOptions,
    TransferOptions,
    NftOptions,
} from '../transfer';

export type __SyncAccountPayloadMethod__ = {
    name: 'SyncAccount';
    data?: AccountSyncOptions;
};

export type __SyncAccountPayload__ = {
    cmd: 'CallAccountMethod';
    payload: {
        accountId: AccountId;
        method: __SyncAccountPayloadMethod__;
    };
};

export type __GetInfoPayloadMethod__ = {
    name: 'GetNodeInfo';
    data: string[];
};

export type __GenerateAddressesPayloadMethod__ = {
    name: 'GenerateAddresses';
    data: {
        amount: number;
    };
};

export type __GenerateAddressesPayload__ = {
    cmd: 'CallAccountMethod';
    payload: {
        accountId: AccountId;
        method: __GenerateAddressesPayloadMethod__;
    };
};

export type __LatestAddressPayloadMethod__ = {
    name: 'GetLatestAddress';
};

export type __LatestAddressPayload__ = {
    cmd: 'CallAccountMethod';
    payload: {
        accountId: AccountId;
        method: __LatestAddressPayloadMethod__;
    };
};

export type __BalancePayloadMethod__ = {
    name: 'GetBalance';
};

export type __BalancePayload__ = {
    cmd: 'CallAccountMethod';
    payload: {
        accountId: AccountId;
        method: __BalancePayloadMethod__;
    };
};

export type __SetClientOptionsPayload__ = {
    cmd: 'SetClientOptions';
    payload: ClientOptions;
};

export type __SetCollectOutputsPayloadMethod__ = {
    name: 'CollectOutputs';
    data: {
        outputIdsToCollect: string[];
    };
};

export type __SetCollectOutputsPayload__ = {
    cmd: 'CallAccountMethod';
    payload: {
        accountId: AccountId;
        method: __SetCollectOutputsPayloadMethod__;
    };
};

export type __GetOutputsWithAdditionalUnlockConditionsPayloadMethod__ = {
    name: 'GetOutputsWithAdditionalUnlockConditions';
    data: {
        outputsToCollect: OutputsToCollect;
    };
};

export type __GetOutputsWithAdditionalUnlockConditionsPayload__ = {
    cmd: 'CallAccountMethod';
    payload: {
        accountId: AccountId;
        method: __GetOutputsWithAdditionalUnlockConditionsPayloadMethod__;
    };
};

export type __ListAddressesPayloadMethod__ = {
    name: 'ListAddresses';
};

export type __ListAddressesPayload__ = {
    cmd: 'CallAccountMethod';
    payload: {
        accountId: AccountId;
        method: __ListAddressesPayloadMethod__;
    };
};

export type __ListAddressesWithBalancePayloadMethod__ = {
    name: 'ListAddressesWithBalance';
};

export type __ListAddressesWithBalancePayload__ = {
    cmd: 'CallAccountMethod';
    payload: {
        accountId: AccountId;
        method: __ListAddressesWithBalancePayloadMethod__;
    };
};

export type __ListOutputsPayloadMethod__ = {
    name: 'ListOutputs';
};

export type __ListOutputsPayload__ = {
    cmd: 'CallAccountMethod';
    payload: {
        accountId: AccountId;
        method: __ListOutputsPayloadMethod__;
    };
};

export type __ListPendingTransactionsPayloadMethod__ = {
    name: 'ListPendingTransactions';
};

export type __ListPendingTransactionsPayload__ = {
    cmd: 'CallAccountMethod';
    payload: {
        accountId: AccountId;
        method: __ListPendingTransactionsPayloadMethod__;
    };
};

export type __ListTransactionsPayloadMethod__ = {
    name: 'ListTransactions';
};

export type __ListTransactionsPayload__ = {
    cmd: 'CallAccountMethod';
    payload: {
        accountId: AccountId;
        method: __ListTransactionsPayloadMethod__;
    };
};

export type __ListUnspentOutputsPayloadMethod__ = {
    name: 'ListUnspentOutputs';
};

export type __ListUnspentOutputsPayload__ = {
    cmd: 'CallAccountMethod';
    payload: {
        accountId: AccountId;
        method: __ListUnspentOutputsPayloadMethod__;
    };
};

export type __MintNativeTokenPayloadMethod__ = {
    name: 'MintNativeToken';
    data: {
        nativeTokenOptions: NativeTokenOptions;
        options: TransferOptions;
    };
};

export type __MintNativeTokenPayload__ = {
    cmd: 'CallAccountMethod';
    payload: {
        accountId: AccountId;
        method: __MintNativeTokenPayloadMethod__;
    };
};

export type __MintNftsPayloadMethod__ = {
    name: 'MintNfts';
    data: {
        nftsOptions: NftOptions;
        options: TransferOptions;
    };
};

export type __MintNftsPayload__ = {
    cmd: 'CallAccountMethod';
    payload: {
        accountId: AccountId;
        method: __MintNftsPayloadMethod__;
    };
};

export type __SendAmountPayloadMethod__ = {
    name: 'SendAmount';
    data: {
        addressesWithAmount: AddressWithAmount[];
        options: TransferOptions;
    };
};

export type __SendAmountPayload__ = {
    cmd: 'CallAccountMethod';
    payload: {
        accountId: AccountId;
        method: __SendAmountPayloadMethod__;
    };
};

export type __SendMicroTransactionPayloadMethod__ = {
    name: 'SendMicroTransaction';
    data: {
        addressesWithMicroAmount: AddressWithMicroAmount[];
        options: TransferOptions;
    };
};

export type __SendMicroTransactionPayload__ = {
    cmd: 'CallAccountMethod';
    payload: {
        accountId: AccountId;
        method: __SendMicroTransactionPayloadMethod__;
    };
};

export type __SendNativeTokensPayloadMethod__ = {
    name: 'SendNativeTokens';
    data: {
        addressesNativeTokens: AddressNativeTokens[];
        options: TransferOptions;
    };
};

export type __SendNativeTokensPayload__ = {
    cmd: 'CallAccountMethod';
    payload: {
        accountId: AccountId;
        method: __SendNativeTokensPayloadMethod__;
    };
};

export type __SendNftPayloadMethod__ = {
    name: 'SendNft';
    data: {
        addressesNftIds: AddressNftId[];
        options: TransferOptions;
    };
};

export type __SendNftPayload__ = {
    cmd: 'CallAccountMethod';
    payload: {
        accountId: AccountId;
        method: __SendNftPayloadMethod__;
    };
};

export type __SendTransferPayloadMethod__ = {
    name: 'SendTransfer';
    data: {
        outputs: OutputData[];
        options: TransferOptions;
    };
};

export type __SendTransferPayload__ = {
    cmd: 'CallAccountMethod';
    payload: {
        accountId: AccountId;
        method: __SendTransferPayloadMethod__;
    };
};

export type __TryCollectOutputsPayloadMethod__ = {
    name: 'TryCollectOutputs';
    data: {
        outputsToCollect: OutputsToCollect;
    };
};

export type __TryCollectOutputsPayload__ = {
    cmd: 'CallAccountMethod';
    payload: {
        accountId: AccountId;
        method: __TryCollectOutputsPayloadMethod__;
    };
};
