require('dotenv').config();

async function run() {
	const { AccountManager, StorageType, RemainderValueStrategy } = require('iota-wallet')
    const manager = new AccountManager({
        storagePath: './test-database',
        storageType: StorageType.Sqlite
    })


    console.log(manager.getAccounts())
    const account = manager.getAccountByAlias('Test')

    console.log('alias', account.alias())
    console.log('available balance', account.balance().available)
    console.log('syncing...')
    
    const synced = await account.sync()
    
    //TODO: Replace with the address of your choice!
	const addr = 'iot1qym3fgn88ry3ekshh67rr6kky88augzvu9x2j5m0cflwmqmel4phwmfh6qs'
	const amount = 1

	const node_res = await synced.send(
		addr,
		amount
    ) 

	console.log(node_res)
}

run()
