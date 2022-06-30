/**
 * This example creates a new database and account
 */

require('dotenv').config();
const { AccountManager, CoinType } = require('@iota/wallet');

async function run() {
    try {
        const accountManagerOptions = {
            storagePath: './alice-database',
            clientOptions: {
                nodes: ['http://localhost:14265'],
            },
            coinType: CoinType.Shimmer,
            secretManager: {
                Stronghold: {
                    snapshotPath: `./wallet.stronghold`,
                    password: `${process.env.SH_PASSWORD}`,
                },
            },
        };
    
        const manager = new AccountManager(accountManagerOptions);

        // Mnemonic only needs to be set the first time
        await manager.storeMnemonic(process.env.MNEMONIC);
        
        // The coin type only needs to be set on the first account
        const account = await manager.createAccount({
            alias: 'Alice',
            coinType: CoinType.IOTA,
        });
        console.log('Account created:', account);

    } catch (error) {
        console.log('Error: ' + error);
    }
    process.exit(0);
}

run();
