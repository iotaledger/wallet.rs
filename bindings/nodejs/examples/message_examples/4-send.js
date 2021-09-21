/**
 * This example sends IOTA Toens to an address.
 */

require('dotenv').config();

async function run() {
    const { AccountManagerForMessages } = require('@iota/wallet');

    const manager = new AccountManagerForMessages({
        storagePath: './alice-database',
    });

    try {
        await manager.setStrongholdPassword(process.env.SH_PASSWORD);

        const account = await manager.getAccount('Alice');
        console.log('Account:', account);

        // Always sync before doing anything with the account
        const synced = await account.sync();
        console.log('Syncing... - ', synced);

        console.log('Available balance', await account.balance());

        //TODO: Replace with the address of your choice!
        const address =
            'atoi1qzt0nhsf38nh6rs4p6zs5knqp6psgha9wsv74uajqgjmwc75ugupx3y7x0r';
        const amount = 1000000;

        const response = await account.send({
            address,
            amount,
            remainder_value_strategy: {
                strategy: 'ReuseAddress',
            },
        });

        console.log(response);

        console.log(
            `Check your message on https://explorer.iota.org/devnet/message/${node_response.id}`,
        );
    } catch (error) {
        console.log('Error: ' + error);
    }
}

run();
