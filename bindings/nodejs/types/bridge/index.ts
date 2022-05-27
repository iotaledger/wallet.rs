import type { AccountId } from '../account';
import type {
    __SyncAccountMethod__,
    __GenerateAddressesMethod__,
    __GetBalanceMethod__,
    __SetCollectOutputsMethod__,
    __GetOutputMethod__,
    __GetOutputsWithAdditionalUnlockConditionsMethod__,
    __GetTransactionMethod__,
    __ListAddressesMethod__,
    __ListAddressesWithUnspentOutputsMethod__,
    __ListOutputsMethod__,
    __ListUnspentOutputsMethod__,
    __ListPendingTransactionsMethod__,
    __ListTransactionsMethod__,
    __MintNativeTokenMethod__,
    __MintNftsMethod__,
    __SendAmountMethod__,
    __SendMicroTransactionMethod__,
    __SendNativeTokensMethod__,
    __SendNftMethod__,
    __SendTransactionMethod__,
    __TryCollectOutputsMethod__,
    __SetAliasMethod__,
} from './account';
import type {
    __GetAccountsMessage__,
    __GetAccountMessage__,
    __GetNodeInfoMessage__,
    __CreateAccountMessage__,
    __SetStrongholdPasswordMessage__,
    __SetStrongholdPasswordClearIntervalMessage__,
    __StoreMnemonicMessage__,
    __BackupMessage__,
    __RestoreBackupMessage__,
    __GenerateMnemonicMessage__,
    __VerifyMnemonicMessage__,
    __DeleteStorageMessage__,
    __StartBackgroundSyncMessage__,
    __StopBackgroundSyncMessage__,
    __RecoverAccountsMessage__,
    __EmitTestEventMessage__,
    __SetClientOptionsMessage__,
    __IsStrongholdPasswordAvailableMessage__,
    __ClearStrongholdPasswordMessage__,
} from './accountManager';

export type __AccountMethod__ =
    | __GetBalanceMethod__
    | __GenerateAddressesMethod__
    | __GetOutputMethod__
    | __GetOutputsWithAdditionalUnlockConditionsMethod__
    | __GetTransactionMethod__
    | __ListAddressesMethod__
    | __ListAddressesWithUnspentOutputsMethod__
    | __ListOutputsMethod__
    | __ListPendingTransactionsMethod__
    | __ListTransactionsMethod__
    | __ListUnspentOutputsMethod__
    | __MintNativeTokenMethod__
    | __MintNftsMethod__
    | __SendAmountMethod__
    | __SendMicroTransactionMethod__
    | __SendNativeTokensMethod__
    | __SendNftMethod__
    | __SendTransactionMethod__
    | __SetAliasMethod__
    | __SetCollectOutputsMethod__
    | __SyncAccountMethod__
    | __TryCollectOutputsMethod__;

export type __CallAccountMethodMessage__ = {
    cmd: 'CallAccountMethod';
    payload: {
        accountId: AccountId;
        method: __AccountMethod__;
    };
};

export type __Message__ =
    | __BackupMessage__
    | __CallAccountMethodMessage__
    | __ClearStrongholdPasswordMessage__
    | __CreateAccountMessage__
    | __DeleteStorageMessage__
    | __EmitTestEventMessage__
    | __GenerateMnemonicMessage__
    | __GetAccountMessage__
    | __GetAccountsMessage__
    | __GetNodeInfoMessage__
    | __IsStrongholdPasswordAvailableMessage__
    | __RecoverAccountsMessage__
    | __RestoreBackupMessage__
    | __StartBackgroundSyncMessage__
    | __StopBackgroundSyncMessage__
    | __SetClientOptionsMessage__
    | __SetStrongholdPasswordMessage__
    | __SetStrongholdPasswordClearIntervalMessage__
    | __StoreMnemonicMessage__
    | __VerifyMnemonicMessage__;
