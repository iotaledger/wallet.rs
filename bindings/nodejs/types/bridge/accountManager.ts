import type {
    AccountId,
    AccountSyncOptions,
    CreateAccountPayload,
} from '../account';
import type { WalletEvent } from '../event';
import type { Auth, ClientOptions } from '../network';

export type __BackupMessage__ = {
    cmd: 'Backup';
    payload: {
        destination: string;
        password: string;
    };
};

export type __Bech32ToHex__ = {
    cmd: 'Bech32ToHex';
    payload: string;
};

export type __ChangeStrongholdPasswordMessage__ = {
    cmd: 'ChangeStrongholdPassword';
    payload: {
        currentPassword: string;
        newPassword: string;
    };
};

export type __ClearStrongholdPasswordMessage__ = {
    cmd: 'ClearStrongholdPassword';
};

export type __CreateAccountMessage__ = {
    cmd: 'CreateAccount';
    payload: CreateAccountPayload;
};

export type __DeleteAccountsAndDatabaseMessage__ = {
    cmd: 'DeleteAccountsAndDatabase';
};

export type __EmitTestEventMessage__ = {
    cmd: 'EmitTestEvent';
    payload: WalletEvent;
};

export type __GenerateMnemonicMessage__ = {
    cmd: 'GenerateMnemonic';
};

export type __GetAccountsMessage__ = {
    cmd: 'GetAccounts';
};

export type __GetAccountMessage__ = {
    cmd: 'GetAccount';
    payload: AccountId;
};

export type __GetLedgerStatusMessage__ = {
    cmd: 'GetLedgerStatus';
};

export type __GetNodeInfoMessage__ = {
    cmd: 'GetNodeInfo';
    payload: {
        url?: string;
        auth?: Auth;
    };
};

export type __HexToBech32__ = {
    cmd: 'HexToBech32';
    payload: {
        hex: string;
        bech32Hrp?: string;
    };
};

export type __IsStrongholdPasswordAvailableMessage__ = {
    cmd: 'IsStrongholdPasswordAvailable';
};

export type __RecoverAccountsMessage__ = {
    cmd: 'RecoverAccounts';
    payload: {
        accountGapLimit: number;
        addressGapLimit: number;
        syncOptions?: AccountSyncOptions;
    };
};

export type __RemoveLatestAccountMessage__ = {
    cmd: 'RemoveLatestAccount';
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

export type __SetStrongholdPasswordMessage__ = {
    cmd: 'SetStrongholdPassword';
    payload: string;
};

export type __SetStrongholdPasswordClearIntervalMessage__ = {
    cmd: 'SetStrongholdPasswordClearInterval';
    payload?: number;
};

export type __StartBackgroundSyncMessage__ = {
    cmd: 'StartBackgroundSync';
    payload: {
        options?: AccountSyncOptions;
        intervalInMilliseconds?: number;
    };
};

export type __StopBackgroundSyncMessage__ = {
    cmd: 'StopBackgroundSync';
};

export type __StoreMnemonicMessage__ = {
    cmd: 'StoreMnemonic';
    payload: string;
};

export type __VerifyMnemonicMessage__ = {
    cmd: 'VerifyMnemonic';
    payload: string;
};
