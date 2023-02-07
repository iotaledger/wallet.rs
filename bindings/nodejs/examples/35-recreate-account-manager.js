/**
 * This example sends a transaction, destroys the AccountManager and recreates it again.
 */

const getUnlockedManager = require('./account-manager');

async function run() {
    try {
        const { initLogger } = require('@iota/wallet');
        initLogger({
            name: './wallet.log',
            levelFilter: 'debug',
            targetExclusions: ["h2", "hyper", "rustls", "message_interface"]
        });

        let manager = await getUnlockedManager();
        await manager.startBackgroundSync();

        const account = await manager.getAccount('Bob');
        console.log('Account:', account);

        await account.sync();

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


        await manager.stopBackgroundSync();
        await manager.clearStrongholdPassword()
        await manager.destroy();

        manager = await getUnlockedManager();
        let accounts = await manager.getAccounts()
        console.log(accounts)

    } catch (error) {
        console.log('Error: ', error);
    }
    process.exit(0);
}

run();
