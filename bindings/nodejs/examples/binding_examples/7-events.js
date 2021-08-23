/**
 * This example shows some events.
 */

require('dotenv').config()

async function run() {
    const { AccountManager } = require('../../lib/index.js');
    const manager = new AccountManager({
        storagePath: './alice-database'
    })
    console.log(process.env.SH_PASSWORD)
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


    const callback = function (err, data) {
        console.log("data:", data)
    }

    manager.listen("BalanceChange", callback);

    // Event listeners would be removed after 30 seconds.
    setTimeout(() => {
        manager.removeEventListeners("BalanceChange");
        console.log("event listeners removed");
    }, 30000);

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
