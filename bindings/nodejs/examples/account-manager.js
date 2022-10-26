require('dotenv').config({ path: '../.env' });
const { AccountManager } = require('@iota/wallet');

async function getUnlockedManager() {
    const manager = new AccountManager({
        storagePath: './alice-database',
        clientOptions: {
            nodes: ['https://api.testnet.shimmer.network']
        },
        coinType: 4219,
        secretManager: {
            mnemonic: process.env.MNEMONIC
        }      
    });
    await manager.setStrongholdPassword(process.env.SH_PASSWORD);
    return manager;
}

module.exports = getUnlockedManager;
