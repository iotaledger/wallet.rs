/**
 * This example sends IOTA Toens to an address.
 */

 require('dotenv').config();

async function run() {
	const { AccountManager } = require('@iota/wallet')
    const manager = new AccountManager({
        storagePath: './alice-database'
    })

    manager.setStrongholdPassword(process.env.SH_PASSWORD)

    const account = manager.getAccount('Alice')
    
    console.log('alias', account.alias())
    console.log('syncing...')
    const synced = await account.sync()
    console.log('available balance', account.balance().available)
    
    //TODO: Replace with the address of your choice!
	const addr = 'atoi1qykf7rrdjzhgynfkw6z7360avhaaywf5a4vtyvvk6a06gcv5y7sksu7n5cs'
	const amount = 10000000

	const node_response = await synced.send(
		addr,
		amount
    ) 

    console.log(`Check your message on https://explorer.iota.org/chrysalis/message/${node_response.id}`)
}

run()
