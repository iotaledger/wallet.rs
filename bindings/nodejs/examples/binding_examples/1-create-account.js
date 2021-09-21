/**
 * This example creates a new database and account
 */

require('dotenv').config();

async function run() {
    const { AccountManager, SignerType } = require('@iota/wallet');

    const manager = new AccountManager({
        storagePath: './alice-database',
    });

    try {
        manager.setStrongholdPassword(process.env.SH_PASSWORD);
        let account;
        try {
            account = manager.getAccount('Alice');
        } catch (e) {
            console.log("Couldn't get account, creating a new one");
        }

        // Create account only if it does not already exist
        if (!account) {
            manager.storeMnemonic(SignerType.Stronghold);
            account = manager.createAccount({
                clientOptions: {
                    node: { url: 'https://api.lb-0.h.chrysalis-devnet.iota.cafe' },
                    localPow: true,
                },
                alias: 'Alice',
            });
            console.log('Account created:', account.id());
        }

        const synced = await account.sync();
        console.log('Synced account', synced);
    } catch (error) {
        console.log('Error: ' + error);
    }
}

run();
