/**
 * This example shows some events.
 */

require('dotenv').config()

async function run() {
	const { AccountManager, addEventListener } = require('@iota/wallet')
    const manager = new AccountManager({
        storagePath: './alice-database'
    })

    manager.setStrongholdPassword(process.env.SH_PASSWORD)

    const account = manager.getAccount('Alice')
    console.log('Account:', account.alias())

    // Always sync before doing anything with the account
    const synced = await account.sync()
    console.log('Syncing...')
    // let address = account.generateAddress()
    
    // get latest address
    let addressObject = account.latestAddress()

    console.log("Address:", addressObject.address)
    
    // Use the Chrysalis Faucet to send testnet tokens to your address:
    console.log("Fill your address with the Faucet: https://faucet.testnet.chrysalis2.com/")


    const callback = function(data) {
        console.log("data:", data)
    }

    addEventListener("BalanceChange", callback)

    // Possible Event Types:
    //
    // ErrorThrown
    // BalanceChange
    // NewTransaction
    // ConfirmationStateChange
    // Reattachment
    // Broadcast
}

run()
