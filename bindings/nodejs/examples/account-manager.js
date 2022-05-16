require('dotenv').config();
const { AccountManager } = require('@iota/wallet');

async function unlockAndReturnManager() {
    const manager = new AccountManager({
        storagePath: './alice-database',
    });
    await manager.setStrongholdPassword(process.env.SH_PASSWORD);
    return manager;
}

module.exports = unlockAndReturnManager;
