/**
 * This example creates a new database and account
 */

require('dotenv').config();

async function run() {
    const { AccountManagerForMessages } = require('@iota/wallet');

    const manager = new AccountManagerForMessages({
        storagePath: './alice-database',
    });

    try {
        await manager.setStrongholdPassword(process.env.SH_PASSWORD);
        await manager.storeMnemonic();

        const account = await manager.createAccount({
            // todo replace with https://api.lb-0.h.chrysalis-devnet.iota.cafe when the new faucet is working
            clientOptions: {
                node: { url: 'https://api.lb-0.h.chrysalis-devnet.iota.cafe' },
                localPow: true,
            },
            alias: 'Alice',
        });
        console.log('Account created:', account);
    } catch (error) {
        console.log('Error: ' + error);
    }
}

run();
