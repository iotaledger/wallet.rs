import type { AccountId } from '../account';
import type {
    __SyncAccountPayloadMethod__,
    __GenerateAddressesPayloadMethod__,
    __BalancePayloadMethod__,
    __SetCollectOutputsPayloadMethod__,
    __GetOutput__,
    __GetOutputsWithAdditionalUnlockConditionsPayloadMethod__,
    __ListAddressesPayloadMethod__,
    __ListAddressesWithUnspentOutputsPayloadMethod__,
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
} from './account';
import type {
    __GetAccountsPayload__,
    __GetAccountPayload__,
    __GetNodeInfoPayload__,
    __CreateAccountPayload__,
    __SetStrongholdPasswordPayload__,
    __StoreMnemonicPayload__,
    __BackupPayload__,
    __RestoreBackupPayload__,
    __GenerateMnemonicPayload__,
    __VerifyMnemonicPayload__,
    __DeleteStoragePayload__,
    __StartBackgroundSyncPayload__,
    __StopBackgroundSyncPayload__,
    __RecoverAccountsPayload__,
    __EmitTestEventPayload__,
    __SetClientOptionsPayload__,
    __IsStrongholdPasswordAvailablePayload__,
    __ClearStrongholdPasswordPayload__,
} from './accountManager';

export type __AccountPayloadMethods__ =
    | __BalancePayloadMethod__
    | __GenerateAddressesPayloadMethod__
    | __GetOutput__
    | __GetOutputsWithAdditionalUnlockConditionsPayloadMethod__
    | __ListAddressesPayloadMethod__
    | __ListAddressesWithUnspentOutputsPayloadMethod__
    | __ListOutputsPayloadMethod__
    | __ListPendingTransactionsPayloadMethod__
    | __ListTransactionsPayloadMethod__
    | __ListUnspentOutputsPayloadMethod__
    | __MintNativeTokenPayloadMethod__
    | __MintNftsPayloadMethod__
    | __SendAmountPayloadMethod__
    | __SendMicroTransactionPayloadMethod__
    | __SendNativeTokensPayloadMethod__
    | __SendNftPayloadMethod__
    | __SendTransferPayloadMethod__
    | __SetCollectOutputsPayloadMethod__
    | __SyncAccountPayloadMethod__
    | __TryCollectOutputsPayloadMethod__;

export type __CallAccountMethodPayload__ = {
    cmd: 'CallAccountMethod';
    payload: {
        accountId: AccountId;
        method: __AccountPayloadMethods__;
    };
};

export type __SendMessagePayload__ =
    | __BackupPayload__
    | __CallAccountMethodPayload__
    | __ClearStrongholdPasswordPayload__
    | __CreateAccountPayload__
    | __DeleteStoragePayload__
    | __EmitTestEventPayload__
    | __GenerateMnemonicPayload__
    | __GetAccountPayload__
    | __GetAccountsPayload__
    | __GetNodeInfoPayload__
    | __IsStrongholdPasswordAvailablePayload__
    | __RecoverAccountsPayload__
    | __RestoreBackupPayload__
    | __StartBackgroundSyncPayload__
    | __StopBackgroundSyncPayload__
    | __SetClientOptionsPayload__
    | __SetStrongholdPasswordPayload__
    | __StoreMnemonicPayload__
    | __VerifyMnemonicPayload__;
