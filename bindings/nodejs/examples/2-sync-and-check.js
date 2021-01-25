require('dotenv').config();

async function run() {
	const { AccountManager, StorageType } = require('iota-wallet')
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

    const addresses = account.listAddresses(true)
    let unspent_addr = '';
    if(addresses.length == 0) {
        unspent_addr = account.generateAddress().address
    } else {
        unspent_addr = addresses[0].address
    }
    console.log('Need a refill? Send it this address:', unspent_addr)
    
    const s_addresses = account.listAddresses(false)
    console.log(s_addresses)
}

run()
