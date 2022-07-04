export interface LedgerNanoSecretManager{}

export interface LedgerNanoSimulatorLedgerNanoSecretManager{}

export interface MnemonicSecretManager {
    Mnemonic: string;
}

export interface StrongholdSecretManager {
    Stronghold: {
        password?: string;
        snapshotPath?: string;
    };
}

export type SecretManager = LedgerNanoSecretManager | LedgerNanoSimulatorLedgerNanoSecretManager | MnemonicSecretManager | StrongholdSecretManager;
