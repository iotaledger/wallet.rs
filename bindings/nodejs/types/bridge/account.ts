import type { AccountSyncOptions } from '../account';
import type { AddressWithAmount, AddressWithMicroAmount, AddressNativeTokens, AddressNftId, AddressGenerationOptions } from '../address';
import type { OutputsToCollect, OutputData } from '../output';
import type {
    NativeTokenOptions,
    TransferOptions,
    NftOptions,
} from '../transfer';

export type __SyncAccountPayloadMethod__ = {
    name: 'SyncAccount',
    data: AccountSyncOptions
}

export type __GenerateAddressesPayloadMethod__ = {
    name: 'GenerateAddresses';
    data: {
        amount: number
        options?: AddressGenerationOptions
    }
}

export type __LatestAddressPayloadMethod__ = {
    name: 'GetLatestAddress';
};

export type __BalancePayloadMethod__ = {
    name: 'GetBalance';
};

export type __SetCollectOutputsPayloadMethod__ = {
    name: 'CollectOutputs';
    data: {
        outputIdsToCollect: string[];
    };
};

export type __GetOutput__ = {
    name: 'GetOutput',
    data: {
        outputId: string
    }
}

export type __GetOutputsWithAdditionalUnlockConditionsPayloadMethod__ = {
    name: 'GetOutputsWithAdditionalUnlockConditions';
    data: {
        outputsToCollect: OutputsToCollect;
    };
};

export type __ListAddressesPayloadMethod__ = {
    name: 'ListAddresses';
};

export type __ListAddressesWithUnspentOutputsPayloadMethod__ = {
    name: 'ListAddressesWithUnspentOutputs';
}

export type __ListOutputsPayloadMethod__ = {
    name: 'ListOutputs';
};

export type __ListPendingTransactionsPayloadMethod__ = {
    name: 'ListPendingTransactions';
};

export type __ListTransactionsPayloadMethod__ = {
    name: 'ListTransactions';
};

export type __ListUnspentOutputsPayloadMethod__ = {
    name: 'ListUnspentOutputs';
};

export type __MintNativeTokenPayloadMethod__ = {
    name: 'MintNativeToken';
    data: {
        nativeTokenOptions: NativeTokenOptions;
        options: TransferOptions;
    };
};

export type __MintNftsPayloadMethod__ = {
    name: 'MintNfts';
    data: {
        nftsOptions: NftOptions;
        options: TransferOptions;
    };
};

export type __SendAmountPayloadMethod__ = {
    name: 'SendAmount';
    data: {
        addressesWithAmount: AddressWithAmount[];
        options: TransferOptions;
    };
};

export type __SendMicroTransactionPayloadMethod__ = {
    name: 'SendMicroTransaction';
    data: {
        addressesWithMicroAmount: AddressWithMicroAmount[];
        options: TransferOptions;
    };
};

export type __SendNativeTokensPayloadMethod__ = {
    name: 'SendNativeTokens';
    data: {
        addressesNativeTokens: AddressNativeTokens[];
        options: TransferOptions;
    };
};

export type __SendNftPayloadMethod__ = {
    name: 'SendNft';
    data: {
        addressesNftIds: AddressNftId[];
        options: TransferOptions;
    };
};

export type __SendTransferPayloadMethod__ = {
    name: 'SendTransfer';
    data: {
        outputs: OutputData[];
        options: TransferOptions;
    };
};

export type __TryCollectOutputsPayloadMethod__ = {
    name: 'TryCollectOutputs';
    data: {
        outputsToCollect: OutputsToCollect;
    }
}
