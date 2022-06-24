import type { OutputTypes } from '@iota/types';
import type { AccountSyncOptions } from '../account';
import type {
    AddressWithAmount,
    AddressWithMicroAmount,
    AddressNativeTokens,
    AddressNftId,
    AddressGenerationOptions,
} from '../address';
import type {
    BuildAliasOutputData,
    BuildBasicOutputData,
    BuildFoundryOutputData,
    BuildNftOutputData,
} from '../buildOutputData';
import type { OutputOptions } from '../outputOptions';
import type { OutputsToClaim } from '../output';
import type { SignedTransactionEssence } from '../signedTransactionEssence';
import type { PreparedTransactionData } from '../preparedTransactionData';
import type {
    NativeTokenOptions,
    TransactionOptions,
    NftOptions,
} from '../transactionOptions';

export type __BuildAliasOutputMethod__ = {
    name: 'BuildAliasOutput';
    data: BuildAliasOutputData;
};

export type __BuildBasicOutputMethod__ = {
    name: 'BuildBasicOutput';
    data: BuildBasicOutputData;
};

export type __BuildFoundryOutputMethod__ = {
    name: 'BuildFoundryOutput';
    data: BuildFoundryOutputData;
};

export type __BuildNftOutputMethod__ = {
    name: 'BuildNftOutput';
    data: BuildNftOutputData;
};

export type __ClaimOutputsMethod__ = {
    name: 'ClaimOutputs';
    data: {
        outputIdsToClaim: string[];
    };
};

export type __ConsolidateOutputsMethod__ = {
    name: 'ConsolidateOutputs';
    data: {
        force: boolean;
        outputConsolidationThreshold?: number;
    };
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

export type __GetOutputMethod__ = {
    name: 'GetOutput';
    data: {
        outputId: string;
    };
};

export type __GetOutputsWithAdditionalUnlockConditionsMethod__ = {
    name: 'GetOutputsWithAdditionalUnlockConditions';
    data: {
        outputsToClaim: OutputsToClaim;
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

export type __MinimumRequiredStorageDepositMethod__ = {
    name: 'MinimumRequiredStorageDeposit';
    data: {
        outputs: OutputTypes[];
    };
};

export type __MintNativeTokenMethod__ = {
    name: 'MintNativeToken';
    data: {
        nativeTokenOptions: NativeTokenOptions;
        options?: TransactionOptions;
    };
};

export type __MintNftsMethod__ = {
    name: 'MintNfts';
    data: {
        nftsOptions: NftOptions[];
        options?: TransactionOptions;
    };
};

export type __PrepareOutputMethod__ = {
    name: 'PrepareOutput';
    data: {
        options: OutputOptions;
        transactionOptions?: TransactionOptions
    }
}

export type __PrepareSendAmountMethod__ = {
    name: 'PrepareSendAmount';
    data: {
        addressWithAmount: AddressWithAmount[];
        options?: TransactionOptions;
    };
};

export type __PrepareTransactionMethod__ = {
    name: 'PrepareTransaction';
    data: {
        outputs: OutputTypes[];
        options?: TransactionOptions;
    };
};

export type __SendAmountMethod__ = {
    name: 'SendAmount';
    data: {
        addressWithAmount: AddressWithAmount[];
        options?: TransactionOptions;
    };
};

export type __SendMicroTransactionMethod__ = {
    name: 'SendMicroTransaction';
    data: {
        addressWithMicroAmount: AddressWithMicroAmount[];
        options?: TransactionOptions;
    };
};

export type __SendNativeTokensMethod__ = {
    name: 'SendNativeTokens';
    data: {
        addressNativeTokens: AddressNativeTokens[];
        options?: TransactionOptions;
    };
};

export type __SendNftMethod__ = {
    name: 'SendNft';
    data: {
        addressNftIds: AddressNftId[];
        options?: TransactionOptions;
    };
};

export type __SendOutputsMethod__ = {
    name: 'SendOutputs';
    data: {
        outputs: OutputTypes[];
        options?: TransactionOptions;
    };
};

export type __SetAliasMethod__ = {
    name: 'SetAlias';
    data: {
        alias: string;
    };
};

export type __SignTransactionEssenceMethod__ = {
    name: 'SignTransactionEssence';
    data: {
        preparedTransactionData: PreparedTransactionData;
    };
};

export type __SubmitAndStoreTransactionMethod__ = {
    name: 'SubmitAndStoreTransaction';
    data: {
        signedTransactionData: SignedTransactionEssence;
    };
};

export type __SyncAccountMethod__ = {
    name: 'SyncAccount';
    data: AccountSyncOptions;
};

export type __TryClaimOutputsMethod__ = {
    name: 'TryClaimOutputs';
    data: {
        outputsToClaim: OutputsToClaim;
    };
};
