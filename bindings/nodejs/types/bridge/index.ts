import type { AccountId } from '../account';
import type {
    __SyncAccountPayloadMethod__,
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
    __StartBackgroundSyncPayload__,
    __StopBackgroundSyncPayload__,
    __RecoverAccountsPayload__,
    __EmitTestEventPayload__,
    
} from './accountManager'

export type __AccountPayloadMethods__ =
    | __BalancePayloadMethod__
    | __GenerateAddressesPayloadMethod__
    | __GetOutputsWithAdditionalUnlockConditionsPayloadMethod__
    | __LatestAddressPayloadMethod__
    | __ListAddressesPayloadMethod__
    | __ListAddressesWithBalancePayloadMethod__
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
    | __TryCollectOutputsPayloadMethod__

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
    | __CreateAccountMessagePayload__
    | __DeleteStorageMessagePayload__
    | __EmitTestEventPayload__
    | __GenerateMnemonicPayload__
    | __GetAccountMessagePayload__
    | __GetAccountsMessagePayload__
    | __GetNodeInfoPayload__
    | __RecoverAccountsPayload__
    | __RestoreBackupPayload__
    | __StartBackgroundSyncPayload__
    | __StopBackgroundSyncPayload__
    | __SetClientOptionsPayload__
    | __SetStrongholdPasswordPayload__
    | __StoreMnemonicPayload__
    | __VerifyMnemonicPayload__;
