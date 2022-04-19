
import type { AccountId, CreateAccountPayload } from '../account';

export type __GetAccountsMessagePayload__ = {
    cmd: 'GetAccounts'
}

export type __GetAccountMessagePayload__ = {
    cmd: 'GetAccount'
    payload: AccountId
}

export type __CreateAccountMessagePayload__ = {
    cmd: 'CreateAccount'
    payload: CreateAccountPayload
}

export type __SetStrongholdPasswordPayload__ = {
    cmd: 'SetStrongholdPassword'
    payload: string;
}

export type __StoreMnemonicPayload__ = {
    cmd: 'StoreMnemonic'
    payload: {
        signerType: {
            type: 'Stronghold'
        },
        mnemonic: string
    };
}

export type __BackupPayload__ = {
    cmd: 'Backup'
    payload: {
        destination: string
        password: string
    };
}

export type __ImportAccountsPayload__ = {
    cmd: 'RestoreBackup'
    payload: {
        backupPath: string
        password: string
    };
}
