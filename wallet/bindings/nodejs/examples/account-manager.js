const path = require('path')
require('dotenv').config({ path: path.resolve(__dirname, '.env') });
const { AccountManager, CoinType } = require('@iota/wallet');

async function getUnlockedManager() {
    if (!process.env.NODE_URL) {
        throw new Error('.env NODE_URL is undefined, see .env.example');
    }
    if (!process.env.SH_PASSWORD) {
        throw new Error('.env SH_PASSWORD is undefined, see .env.example');
    }

    const manager = new AccountManager({
        storagePath: './alice-database',
        clientOptions: {
            nodes: [process.env.NODE_URL],
            localPow: true,
        },
        coinType: CoinType.Shimmer,
        secretManager: {
            Stronghold: {
                snapshotPath: `./wallet.stronghold`,
                password: `${process.env.SH_PASSWORD}`,
            },
        },
    });
    return manager;
}

module.exports = getUnlockedManager;
