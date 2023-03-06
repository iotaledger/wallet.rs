const path = require('path')
require('dotenv').config({ path: path.resolve(__dirname, '.env') });
const { AccountManager, CoinType } = require('@iota/wallet');

// In this example we will create addresses with a ledger nano hardware wallet
// To use the ledger nano simulator clone https://github.com/iotaledger/ledger-shimmer-app, run `git submodule init && git submodule update --recursive`,
// then `./build.sh -m nanos|nanox|nanosplus -s` and use `true` for `LedgerNano`.

async function run() {
    try {
        const { initLogger } = require('@iota/wallet');
        initLogger({
            name: './wallet.log',
            levelFilter: 'debug',
            targetExclusions: ["h2", "hyper", "rustls"]
        });

        const manager = new AccountManager({
            storagePath: './alice-database',
            clientOptions: {
                nodes: ['https://api.testnet.shimmer.network'],
            },
            coinType: CoinType.Shimmer,
            secretManager: {
                ledgerNano: true,
            },
        });

        const account = await manager.createAccount({
            alias: 'Ledger',
        });
        console.log('Account:', account);

        const address = await account.generateAddress();
        console.log('New address:', address);

    } catch (error) {
        console.log('Error: ', error);
    }
    process.exit(0);
}

run();
