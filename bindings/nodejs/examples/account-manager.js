require('dotenv').config({ path: '../.env' });
const { AccountManager } = require('@iota/wallet');

async function getUnlockedManager() {
    const manager = new AccountManager({
        storagePath: './alice-database',
    });
    await manager.setStrongholdPassword(process.env.SH_PASSWORD);
    return manager;
}

module.exports = getUnlockedManager;
