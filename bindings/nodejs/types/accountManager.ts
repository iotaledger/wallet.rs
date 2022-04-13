
import type { ClientOptions } from './network';
import type { Signer } from './signer';

export interface AccountManagerOptions {
    storagePath?: string;
    clientOptions: ClientOptions;
    signer: Signer;
}

