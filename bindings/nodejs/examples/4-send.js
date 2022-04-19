/**
 * This example sends IOTA Toens to an address.
 */

require('dotenv').config();
const manager = require('./account-manager');

async function run() {
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
            `Check your message on http://localhost:14265/api/v2/messages/${response.id}`,
        );
    } catch (error) {
        console.log('Error: ' + error);
    }
    process.exit(0);
}

run();
