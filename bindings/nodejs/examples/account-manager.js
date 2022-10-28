require('dotenv').config({ path: '../.env' });
const { AccountManager, CoinType } = require('@iota/wallet');

async function getUnlockedManager() {
    const manager = new AccountManager({
        storagePath: './alice-database',
        clientOptions: {
            nodes: ['https://api.testnet.shimmer.network'],
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
    await manager.setStrongholdPassword(process.env.SH_PASSWORD);
    return manager;
}

module.exports = getUnlockedManager;
