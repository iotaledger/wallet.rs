export interface LedgerNanoSecretManager {
    isSimulator: boolean;
}

export interface MnemonicSecretManager {
    Mnemonic: string;
}

export interface StrongholdSecretManager {
    Stronghold: {
        password?: string;
        snapshotPath?: string;
    };
}

export interface LedgerStatus {
    connected: boolean;
    locked: boolean;
    blindSigningEnabled: boolean;
    app?: LedgerApp;
    device?: LedgerDeviceType;
    bufferSize?: number;
}

export interface LedgerApp {
    name: string;
    version: string;
}

export enum LedgerDeviceType {
    LedgerNanoS = 'LedgerNanoS',
    LedgerNanoX = 'LedgerNanoX',
    LedgerNanoSPlus = 'LedgerNanoSPlus',
}

export type SecretManager =
    | LedgerNanoSecretManager
    | MnemonicSecretManager
    | StrongholdSecretManager;
