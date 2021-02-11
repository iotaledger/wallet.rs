async function run() {
	const { AccountManager, StorageType } = require('@iota/wallet')
    const manager = new AccountManager({
        storagePath: './alice-database',
        storageType: StorageType.Stronghold
    })

    manager.setStrongholdPassword('your_password')

    const account = manager.getAccount('Alice')
    
    console.log('Account:', account.alias())
    
    // Always sync before doing anything with the account
    const synced = await account.sync()
    console.log('Syncing...')

    console.log('Available balance', account.balance().available)
}

run()
