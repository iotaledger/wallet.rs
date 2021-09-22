/**
 * This example restores a secured backup file.
 */

require('dotenv').config();

async function run() {
    const { AccountManager } = require('@iota/wallet');

    const manager = new AccountManager({
        storagePath: './alice-database',
    });

    // Add the path to the file from example 5-backup.js
    // for example: ./backup/2021-02-12T01-23-11-iota-wallet-backup-wallet.stronghold
    const path = 'input your backup file';

    manager.importAccounts(path, process.env.SH_PASSWORD);
    const account = manager.getAccount('Alice');
    console.log('Account:', account.alias());
}

run();
