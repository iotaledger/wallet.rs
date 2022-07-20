/**
 * This example sends IOTA micro transactions to an address.
 */

const getUnlockedManager = require('./account-manager');

async function run() {
    try {
        const manager = await getUnlockedManager();
        const account = await manager.getAccount('Alice');

        await account.sync();

        //TODO: Replace with the address of your choice!
        const address =
            'rms1qrrv7flg6lz5cssvzv2lsdt8c673khad060l4quev6q09tkm9mgtupgf0h0';
        const amount = '1000';

        const response = await account.sendMicroTransaction([
            {
                address,
                amount,
            },
        ]);

        console.log(response);

        console.log(
            `Check your block on http://localhost:14265/api/core/v2/blocks/${response.blockId}`,
        );
    } catch (error) {
        console.log('Error: ', error);
    }
    process.exit(0);
}

run();
