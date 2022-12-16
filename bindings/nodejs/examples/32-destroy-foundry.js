/**
 * This example will burn native tokens
 */
const getUnlockedManager = require('./account-manager');

async function run() {
    try {
        const manager = await getUnlockedManager();

        const account = await manager.getAccount('0');

        await account.sync();

        // Get a foundry id from your account balance after running example
        // 22-mint-native-tokens.js
        let foundryId =
            '0x08e6210d29881310db2afde095e594f6f006fcdbd06e7a83b74bd2bdf3b5190d0e0200000000';

        const response = await account.destroyFoundry(foundryId);

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
