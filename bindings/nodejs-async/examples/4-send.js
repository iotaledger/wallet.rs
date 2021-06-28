/**
 * This example sends IOTA Toens to an address.
 */

 async function run() {
    const { MessageHandler } = require('../lib/index.js');
    const manager = new MessageHandler({
        storagePath: './alice-database',
    });
    try {
        await manager.setStrongholdPassword("A12345678*");

        const account = await manager.getAccount('Alice')
        console.log('Account:', account)

        // Always sync before doing anything with the account
        const synced = await account.sync()
        console.log('Syncing... - ' + synced)

        console.log('Available balance', await account.balance())

        //TODO: Replace with the address of your choice!
        const address = 'atoi1qq4yjuenvm63sklmztfseqxa90xeumxnx4rt79j5vstwqqy0jn9lujjnmh0'
        const amount = 10000000

        const node_response = await account.send({
                address,
                amount,
                remainder_value_strategy: {
                    strategy: 'ReuseAddress',
                },
        });
        console.log(node_response);

        console.log(`Check your message on https://explorer.iota.org/chrysalis/message/${node_response.id}`)
    } catch (error) {
        console.log("Error: " + error)
    }
}

run()
