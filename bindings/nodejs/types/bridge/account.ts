import type { OutputTypes, HexEncodedAmount } from '@iota/types';
import type { AccountSyncOptions, FilterOptions } from '../account';
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
    MintMoreNativeTokenOptions,
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

export type __BurnNativeTokenMethod__ = {
    name: 'BurnNativeToken';
    data: {
        nativeToken: [string, HexEncodedAmount];
        options?: TransactionOptions;
    };
};

export type __BurnNftMethod__ = {
    name: 'BurnNft';
    data: {
        nftId: string;
        options?: TransactionOptions;
    };
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

export type __DestroyAliasMethod__ = {
    name: 'DestroyAlias';
    data: {
        aliasId: string;
        options?: TransactionOptions;
    };
};

export type __DestroyFoundryMethod__ = {
    name: 'DestroyFoundry';
    data: {
        foundryId: string;
        options?: TransactionOptions;
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

export type __GetFoundryOutputMethod__ = {
    name: 'GetFoundryOutput';
    data: {
        tokenId: string;
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
    data: {
        filterOptions?: FilterOptions;
    };
};

export type __ListPendingTransactionsMethod__ = {
    name: 'ListPendingTransactions';
};

export type __ListTransactionsMethod__ = {
    name: 'ListTransactions';
};

export type __ListUnspentOutputsMethod__ = {
    name: 'ListUnspentOutputs';
    data: {
        filterOptions?: FilterOptions;
    };
};

export type __MeltNativeTokenMethod__ = {
    name: 'MeltNativeToken';
    data: {
        nativeToken: [string, HexEncodedAmount];
        options?: TransactionOptions;
    };
};

export type __MinimumRequiredStorageDepositMethod__ = {
    name: 'MinimumRequiredStorageDeposit';
    data: {
        output: OutputTypes;
    };
};

export type __MintMoreNativeTokenMethod__ = {
    name: 'MintMoreNativeToken';
    data: {
        mintMoreNativeTokenOptions: MintMoreNativeTokenOptions;
        options?: TransactionOptions;
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
        transactionOptions?: TransactionOptions;
    };
};

export type __PrepareSendAmountMethod__ = {
    name: 'PrepareSendAmount';
    data: {
        addressesWithAmount: AddressWithAmount[];
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
        addressesWithAmount: AddressWithAmount[];
        options?: TransactionOptions;
    };
};

export type __SendMicroTransactionMethod__ = {
    name: 'SendMicroTransaction';
    data: {
        addressesWithMicroAmount: AddressWithMicroAmount[];
        options?: TransactionOptions;
    };
};

export type __SendNativeTokensMethod__ = {
    name: 'SendNativeTokens';
    data: {
        addressesNativeTokens: AddressNativeTokens[];
        options?: TransactionOptions;
    };
};

export type __SendNftMethod__ = {
    name: 'SendNft';
    data: {
        addressesAndNftIds: AddressNftId[];
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
    data: {
        options?: AccountSyncOptions;
    };
};

export type __TryClaimOutputsMethod__ = {
    name: 'TryClaimOutputs';
    data: {
        outputsToClaim: OutputsToClaim;
    };
};
