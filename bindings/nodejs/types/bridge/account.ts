import type { AccountSyncOptions } from '../account';
import type {
    AddressWithAmount,
    AddressWithMicroAmount,
    AddressNativeTokens,
    AddressNftId,
    AddressGenerationOptions,
} from '../address';
import type { OutputsToCollect, OutputData } from '../output';
import type {
    NativeTokenOptions,
    TransferOptions,
    NftOptions,
} from '../transfer';

export type __SyncAccountMethod__ = {
    name: 'SyncAccount';
    data: AccountSyncOptions;
};

export type __GenerateAddressesMethod__ = {
    name: 'GenerateAddresses';
    data: {
        amount: number;
        options?: AddressGenerationOptions;
    };
};

export type __GetBalanceMethod__ = {
    name: 'GetBalance';
};

export type __SetCollectOutputsMethod__ = {
    name: 'CollectOutputs';
    data: {
        outputIdsToCollect: string[];
    };
};

export type __GetOutputMethod__ = {
    name: 'GetOutput';
    data: {
        outputId: string;
    };
};

export type __GetOutputsWithAdditionalUnlockConditionsMethod__ = {
    name: 'GetOutputsWithAdditionalUnlockConditions';
    data: {
        outputsToCollect: OutputsToCollect;
    };
};

export type __GetTransactionMethod__ = {
    name: 'GetTransaction';
    data: {
        transactionId: string;
    };
};

export type __ListAddressesMethod__ = {
    name: 'ListAddresses';
};

export type __ListAddressesWithUnspentOutputsMethod__ = {
    name: 'ListAddressesWithUnspentOutputs';
};

export type __ListOutputsMethod__ = {
    name: 'ListOutputs';
};

export type __ListPendingTransactionsMethod__ = {
    name: 'ListPendingTransactions';
};

export type __ListTransactionsMethod__ = {
    name: 'ListTransactions';
};

export type __ListUnspentOutputsMethod__ = {
    name: 'ListUnspentOutputs';
};

export type __MintNativeTokenMethod__ = {
    name: 'MintNativeToken';
    data: {
        nativeTokenOptions: NativeTokenOptions;
        options?: TransferOptions;
    };
};

export type __MintNftsMethod__ = {
    name: 'MintNfts';
    data: {
        nftsOptions: NftOptions[];
        options?: TransferOptions;
    };
};

export type __SendAmountMethod__ = {
    name: 'SendAmount';
    data: {
        addressWithAmount: AddressWithAmount[];
        options?: TransferOptions;
    };
};

export type __SendMicroTransactionMethod__ = {
    name: 'SendMicroTransaction';
    data: {
        addressWithMicroAmount: AddressWithMicroAmount[];
        options?: TransferOptions;
    };
};

export type __SendNativeTokensMethod__ = {
    name: 'SendNativeTokens';
    data: {
        addressNativeTokens: AddressNativeTokens[];
        options?: TransferOptions;
    };
};

export type __SendNftMethod__ = {
    name: 'SendNft';
    data: {
        addressNftIds: AddressNftId[];
        options?: TransferOptions;
    };
};

export type __SendTransferMethod__ = {
    name: 'SendTransfer';
    data: {
        outputs: OutputData[];
        options?: TransferOptions;
    };
};

export type __TryCollectOutputsMethod__ = {
    name: 'TryCollectOutputs';
    data: {
        outputsToCollect: OutputsToCollect;
    };
};
