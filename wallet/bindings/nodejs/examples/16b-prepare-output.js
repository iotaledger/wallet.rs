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
        const tag = '0x12345678'
        const metadata = '0x00000000025e4b3ca1e3f42320a10700000000000200000001006115000000038a323cf02b18a59e209c66817a057a1000e0b71201006301000000ff0040420f0000000000020000000000'
        const output = await account.prepareOutput({
            recipientAddress,
            amount,
            features: {
                tag,
                metadata
            }
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
