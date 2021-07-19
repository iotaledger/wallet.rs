/**
 * This example restores a secured backup file.
 */

 async function run() {
    const { AccountManager } = require('../lib/index.js');
    const manager = new AccountManager({
        storagePath: './alice-database',
    });
    try {
        await manager.setStrongholdPassword("A12345678*");

    // Add the path to the file from example 5-backup.js
    // for example: ./backup/2021-02-12T01-23-11-iota-wallet-backup-wallet.stronghold
    let backup_path = "./backup";//"input your backup file"

    await manager.importAccounts(backup_path, "A12345678*")
    const account = manager.getAccount('Alice')
    console.log('Account:', account.alias())
    } catch (error) {
        console.log("Error: " + error)
    }
}

run()
