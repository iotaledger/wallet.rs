/**
 * This example restores a secured backup file. 
 */

async function run() {

    const { AccountManager, StorageType } = require('@iota/wallet')
    const manager = new AccountManager({
        storagePath: './alice-database',
        storageType: StorageType.Stronghold
    })

    // Add the path from example 5-backup.js
    // for example: ./backup/2021-02-12T01-23-11-iota-wallet-backup-wallet.stronghold
    let backup_path = "input your backup path"
    let password = "your_password"

    await manager.importAccounts(backup_path, password)
    const account = manager.getAccount('Alice')
    console.log('Account:', account.alias())
}

run()