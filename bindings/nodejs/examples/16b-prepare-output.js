/**
 * This example builds an output.
 */

const getUnlockedManager = require('./account-manager');

async function run() {
    try {
        const manager = await getUnlockedManager();
        const account = await manager.getAccount('Alice');

        //TODO: Replace with the address of your choice!
        const recipientAddress =
            'rms1qrrv7flg6lz5cssvzv2lsdt8c673khad060l4quev6q09tkm9mgtupgf0h0';
        const amount = '1000';

        const output = await account.prepareOutput({
            recipientAddress,
            amount,
        });
        const minimumRequiredStorageDeposit =
            await account.minimumRequiredStorageDeposit(output);

        console.log('Output:', output);
        console.log(
            'Minimum required storage deposit:',
            minimumRequiredStorageDeposit,
        );
    } catch (error) {
        console.log('Error: ', error);
    }
    process.exit(0);
}

run();
