/**
 * This example sends IOTA Toens to an address.
 */

const getUnlockedManager = require('./account-manager');

async function run() {
    try {
        const manager = await getUnlockedManager();
        const account = await manager.getAccount('Alice');
        console.log('Account:', account);

        // Always sync before doing anything with the account
        const synced = await account.sync();
        console.log('Syncing... - ', synced);

        console.log('Available balance', await account.getBalance());

        //TODO: Replace with the address of your choice!
        const address =
            'rms1qrrv7flg6lz5cssvzv2lsdt8c673khad060l4quev6q09tkm9mgtupgf0h0';
        const amount = '1000000';

        const response = await account.sendAmount([
            {
                address,
                amount,
            },
        ]);

        console.log(response);

        console.log(
            `Check your message on http://localhost:14265/api/v2/messages/${response.messageId}`,
        );
    } catch (error) {
        console.log('Error: ' + error);
    }
    process.exit(0);
}

run();
