import type { AccountId } from '../account';
import type {
    __SyncAccountPayloadMethod__,
    __GetInfoPayloadMethod__,
    __GenerateAddressesPayloadMethod__,
    __LatestAddressPayloadMethod__,
    __BalancePayloadMethod__,
    __SetCollectOutputsPayloadMethod__,
    __GetOutputsWithAdditionalUnlockConditionsPayloadMethod__,
    __ListAddressesPayloadMethod__,
    __ListAddressesWithBalancePayloadMethod__,
    __ListOutputsPayloadMethod__,
    __ListUnspentOutputsPayloadMethod__,
    __ListPendingTransactionsPayloadMethod__,
    __ListTransactionsPayloadMethod__,
    __MintNativeTokenPayloadMethod__,
    __MintNftsPayloadMethod__,
    __SendAmountPayloadMethod__,
    __SendMicroTransactionPayloadMethod__,
    __SendNativeTokensPayloadMethod__,
    __SendNftPayloadMethod__,
    __SendTransferPayloadMethod__,
    __TryCollectOutputsPayloadMethod__,
    __SyncAccountPayload__,
    __GenerateAddressesPayload__,
    __LatestAddressPayload__,
    __BalancePayload__,
    __SetClientOptionsPayload__,
    __SetCollectOutputsPayload__,
    __GetOutputsWithAdditionalUnlockConditionsPayload__,
    __ListAddressesPayload__,
    __ListAddressesWithBalancePayload__,
    __ListOutputsPayload__,
    __ListPendingTransactionsPayload__,
    __ListTransactionsPayload__,
    __ListUnspentOutputsPayload__,
    __MintNativeTokenPayload__,
    __MintNftsPayload__,
    __SendAmountPayload__,
    __SendMicroTransactionPayload__,
    __SendNativeTokensPayload__,
    __SendNftPayload__,
    __SendTransferPayload__,
    __TryCollectOutputsPayload__,
} from './account';
import type {
    __GetAccountsMessagePayload__,
    __GetAccountMessagePayload__,
    __GetNodeInfoPayload__,
    __CreateAccountMessagePayload__,
    __SetStrongholdPasswordPayload__,
    __StoreMnemonicPayload__,
    __BackupPayload__,
    __RestoreBackupPayload__,
    __GenerateMnemonicPayload__,
    __VerifyMnemonicPayload__,
    __DeleteStorageMessagePayload__,
    
} from './accountManager'

export type __AccountPayloadMethods__ =
    | __SyncAccountPayloadMethod__
    | __GetInfoPayloadMethod__
    | __GenerateAddressesPayloadMethod__
    | __LatestAddressPayloadMethod__
    | __BalancePayloadMethod__
    | __SetCollectOutputsPayloadMethod__
    | __GetOutputsWithAdditionalUnlockConditionsPayloadMethod__
    | __ListAddressesPayloadMethod__
    | __ListAddressesWithBalancePayloadMethod__
    | __ListOutputsPayloadMethod__
    | __ListUnspentOutputsPayloadMethod__
    | __ListPendingTransactionsPayloadMethod__
    | __ListTransactionsPayloadMethod__
    | __MintNativeTokenPayloadMethod__
    | __MintNftsPayloadMethod__
    | __SendAmountPayloadMethod__
    | __SendMicroTransactionPayloadMethod__
    | __SendNativeTokensPayloadMethod__
    | __SendNftPayloadMethod__
    | __SendTransferPayloadMethod__
    | __TryCollectOutputsPayloadMethod__;

export type __CallAccountMethodPayload__ = {
    cmd: 'CallAccountMethod';
    payload: {
        accountId: AccountId;
        method: __AccountPayloadMethods__;
    };
};

export type __SendMessagePayload__ =
    | __GetAccountsMessagePayload__
    | __GetAccountMessagePayload__
    | __CreateAccountMessagePayload__
    | __DeleteStorageMessagePayload__
    | __CallAccountMethodPayload__
    | __SetStrongholdPasswordPayload__
    | __GenerateMnemonicPayload__
    | __StoreMnemonicPayload__
    | __VerifyMnemonicPayload__
    | __BackupPayload__
    | __RestoreBackupPayload__
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
