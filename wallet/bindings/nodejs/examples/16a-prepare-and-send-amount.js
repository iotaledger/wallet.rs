/**
 * This example prepares, signs and stores a transaction transferring base coins in wallet.rs
 */
const getUnlockedManager = require('./account-manager');

async function run() {
    try {
        const manager = await getUnlockedManager();
        const account = await manager.getAccount('0');

        await account.sync();
        
        const address =
        'rms1qrrv7flg6lz5cssvzv2lsdt8c673khad060l4quev6q09tkm9mgtupgf0h0';
        const amount = '1000000';

        const preparedTransaction = await account.prepareSendAmount([
            {
                address,
                amount,
            },
        ]);

        const signedTransactionEssence = await account.signTransactionEssence(preparedTransaction)
        const response = await account.submitAndStoreTransaction(signedTransactionEssence)
        
        console.log(response)
    } catch (error) {
        console.log('Error: ', error);
    }
    process.exit(0);
}

run();
