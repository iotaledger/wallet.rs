/**
 * This example backups your data in a secure file. 
 * You can move this file to another app or device and restore it.
 */

async function run() {

    const { AccountManager, StorageType } = require('@iota/wallet')
    const manager = new AccountManager({
        storagePath: './alice-database',
        storageType: StorageType.Stronghold
    })

    manager.setStrongholdPassword('your_password')

    let backup_path = await manager.backup("./backup")
    
    console.log('Backup path:', backup_path)
}

run()
