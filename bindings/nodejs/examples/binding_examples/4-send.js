/**
 * This example sends IOTA Toens to an address.
 */

require('dotenv').config();

async function run() {
    const { AccountManager, RemainderValueStrategy } = require('../../lib/index.js');
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
    const addr = 'atoi1qzt0nhsf38nh6rs4p6zs5knqp6psgha9wsv74uajqgjmwc75ugupx3y7x0r'
    const amount = 1000000

    const node_response = await account.send(
        addr,
        amount,
        { remainderValueStrategy: RemainderValueStrategy.reuseAddress() }
    )

    console.log(`Check your message on https://explorer.iota.org/chrysalis/message/${node_response.id}`)
}

run()
