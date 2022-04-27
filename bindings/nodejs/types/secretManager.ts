export interface MnemonicSecretManager {
    Mnemonic: string;
}

export interface StrongholdSecretManager {
    Stronghold: {
        password: string;
        snapshotPath: string;
    }
}

export type SecretManager = MnemonicSecretManager | StrongholdSecretManager;
