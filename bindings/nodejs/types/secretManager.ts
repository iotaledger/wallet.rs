export interface MnemonicSecretManager {
    Mnemonic: string;
}

export interface Strongholdsecret_manager {
    Stronghold: {
        password: string;
        snapshotPath: string;
    }
}

export type SecretManager = MnemonicSecretManager | Strongholdsecret_manager;
