/**
 * This example builds an output.
 */

const getUnlockedManager = require('./account-manager');

async function run() {
    try {
        const manager = await getUnlockedManager();
        const account = await manager.getAccount('Alice');
        const recipientAddress =
            'rms1qrrv7flg6lz5cssvzv2lsdt8c673khad060l4quev6q09tkm9mgtupgf0h0';
        const amount = '1000';

        const output = await account.prepareOutput({
            recipientAddress,
            amount,
            features:  {
                tag: 'Random tag',
                metadata: 'Random Data',
            }
        })
        
        const response = await account.sendOutputs([output])
        const transaction = await account.getTransaction(response.transactionId)
        const outputs = transaction.payload.essence.outputs[0]
        console.log('Tag and metadata should show as string below:')
        console.log(outputs)
    } catch (error) {
        console.log('Error: ' + error);
    }
}

run();
