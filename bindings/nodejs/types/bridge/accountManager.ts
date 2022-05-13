import type { AccountId, CreateAccountPayload } from '../account';
import type { Auth } from '../network';

export type __GetAccountsMessagePayload__ = {
    cmd: 'GetAccounts';
};

export type __GetAccountMessagePayload__ = {
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

export type __CreateAccountMessagePayload__ = {
    cmd: 'CreateAccount';
    payload: CreateAccountPayload;
};

export type __DeleteStorageMessagePayload__ = {
    cmd: 'DeleteStorage'
}

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
    cmd: 'RestoreBackup'
    payload: {
        source: string
        password: string
    };
};
