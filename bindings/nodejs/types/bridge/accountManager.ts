import type {
    AccountId,
    AccountSyncOptions,
    CreateAccountPayload,
} from '../account';
import type { WalletEvent } from '../event';
import type { Auth, ClientOptions } from '../network';

export type __GetAccountsMessage__ = {
    cmd: 'GetAccounts';
};

export type __GetAccountMessage__ = {
    cmd: 'GetAccount';
    payload: AccountId;
};

export type __GetNodeInfoMessage__ = {
    cmd: 'GetNodeInfo';
    payload: {
        url?: string;
        auth?: Auth;
    };
};

export type __CreateAccountMessage__ = {
    cmd: 'CreateAccount';
    payload: CreateAccountPayload;
};

export type __RecoverAccountsMessage__ = {
    cmd: 'RecoverAccounts';
    payload: {
        accountGapLimit: number;
        addressGapLimit: number;
    };
};

export type __DeleteStorageMessage__ = {
    cmd: 'DeleteStorage';
};

export type __EmitTestEventMessage__ = {
    cmd: 'EmitTestEvent';
    payload: WalletEvent;
};

export type __ClearStrongholdPasswordMessage__ = {
    cmd: 'ClearStrongholdPassword';
};

export type __IsStrongholdPasswordAvailableMessage__ = {
    cmd: 'IsStrongholdPasswordAvailable';
};

export type __SetStrongholdPasswordMessage__ = {
    cmd: 'SetStrongholdPassword';
    payload: string;
};

export type __GenerateMnemonicMessage__ = {
    cmd: 'GenerateMnemonic';
};

export type __StoreMnemonicMessage__ = {
    cmd: 'StoreMnemonic';
    payload: string;
};

export type __VerifyMnemonicMessage__ = {
    cmd: 'VerifyMnemonic';
    payload: string;
};

export type __BackupMessage__ = {
    cmd: 'Backup';
    payload: {
        destination: string;
        password: string;
    };
};

export type __RestoreBackupMessage__ = {
    cmd: 'RestoreBackup';
    payload: {
        source: string;
        password: string;
    };
};

export type __SetClientOptionsMessage__ = {
    cmd: 'SetClientOptions';
    payload: ClientOptions;
};

export type __StartBackgroundSyncMessage__ = {
    cmd: 'StartBackgroundSync';
    payload: {
        options?: AccountSyncOptions;
        interval?: number;
    };
};

export type __StopBackgroundSyncMessage__ = {
    cmd: 'StopBackgroundSync';
};
