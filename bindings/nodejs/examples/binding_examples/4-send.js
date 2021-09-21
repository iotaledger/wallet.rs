/**
 * This example sends IOTA tokens to an address.
 */

require('dotenv').config();

async function run() {
    const {
        AccountManager,
        RemainderValueStrategy,
    } = require('@iota/wallet');

    const manager = new AccountManager({
        storagePath: './alice-database',
    });

    manager.setStrongholdPassword(process.env.SH_PASSWORD);

    const account = manager.getAccount('Alice');

    console.log('Alias', account.alias());
    console.log('Syncing...');
    await account.sync();
    console.log('Available balance', account.balance().available);

    //TODO: Replace with the address of your choice!
    const address =
        'atoi1qzt0nhsf38nh6rs4p6zs5knqp6psgha9wsv74uajqgjmwc75ugupx3y7x0r';
    const amount = 1000000;

    const response = await account.send(address, amount, {
        remainderValueStrategy: RemainderValueStrategy.reuseAddress(),
    });

    console.log(
        `Check your message on https://explorer.iota.org/devnet/message/${response.id}`,
    );
}

run();
