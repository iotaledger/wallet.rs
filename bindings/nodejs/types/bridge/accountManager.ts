import type {
    AccountId,
    AccountSyncOptions,
    CreateAccountPayload,
} from '../account';
import type { WalletEvent } from '../event';
import type { Auth, ClientOptions } from '../network';

export type __GetAccountsPayload__ = {
    cmd: 'GetAccounts';
};

export type __GetAccountPayload__ = {
    cmd: 'GetAccount';
    payload: AccountId;
};

export type __GetNodeInfoPayload__ = {
    cmd: 'GetNodeInfo';
    payload: {
        url?: string;
        auth?: Auth;
    };
};

export type __CreateAccountPayload__ = {
    cmd: 'CreateAccount';
    payload: CreateAccountPayload;
};

export type __RecoverAccountsPayload__ = {
    cmd: 'RecoverAccounts';
    payload: {
        accountGapLimit: number;
        addressGapLimit: number;
    };
};

export type __DeleteStoragePayload__ = {
    cmd: 'DeleteStorage';
};

export type __EmitTestEventPayload__ = {
    cmd: 'EmitTestEvent';
    payload: WalletEvent;
};

export type __ClearStrongholdPasswordPayload__ = {
    cmd: 'ClearStrongholdPassword';
};

export type __IsStrongholdPasswordAvailablePayload__ = {
    cmd: 'IsStrongholdPasswordAvailable';
};

export type __SetStrongholdPasswordPayload__ = {
    cmd: 'SetStrongholdPassword';
    payload: string;
};

export type __GenerateMnemonicPayload__ = {
    cmd: 'GenerateMnemonic';
};

export type __StoreMnemonicPayload__ = {
    cmd: 'StoreMnemonic';
    payload: string;
};

export type __VerifyMnemonicPayload__ = {
    cmd: 'VerifyMnemonic';
    payload: string;
};

export type __BackupPayload__ = {
    cmd: 'Backup';
    payload: {
        destination: string;
        password: string;
    };
};

export type __RestoreBackupPayload__ = {
    cmd: 'RestoreBackup';
    payload: {
        source: string;
        password: string;
    };
};

export type __SetClientOptionsPayload__ = {
    cmd: 'SetClientOptions';
    payload: ClientOptions;
};

export type __StartBackgroundSyncPayload__ = {
    cmd: 'StartBackgroundSync';
    payload: {
        options?: AccountSyncOptions;
        interval?: number;
    };
};

export type __StopBackgroundSyncPayload__ = {
    cmd: 'StopBackgroundSync';
};
