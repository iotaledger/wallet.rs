export interface MnemonicSigner {
    Mnemonic: string;
}

export interface StrongholdSigner {
    Stronghold: {
        password: string;
        snapshotPath: string;
    }
}

export type Signer = MnemonicSigner | StrongholdSigner;
