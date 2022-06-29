import type { CoinType } from './account';
import type { ClientOptions } from './network';
import type { SecretManager } from './secretManager';

export interface AccountManagerOptions {
    storagePath?: string;
    clientOptions?: ClientOptions;
    coinType?: CoinType;
    secretManager?: SecretManager;
}
