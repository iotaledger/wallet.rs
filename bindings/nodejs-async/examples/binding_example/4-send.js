/**
 * This example sends IOTA Toens to an address.
 */

 require('dotenv').config();

async function run() {
    const { AccountManager } = require('../../lib/index.js');
    const utils = require('../../lib/utils.js');
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
    console.log(utils.RemainderValueStrategy.reuseAddress());
    const node_response = await account.send(
        addr,
        amount,
        {remainderValueStrategy: utils.RemainderValueStrategy.reuseAddress()}
    )

    console.log(`Check your message on https://explorer.iota.org/chrysalis/message/${node_response.id}`)
}

run()
