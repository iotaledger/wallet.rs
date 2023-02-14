/**
 * This example will melt native tokens
 */
const getUnlockedManager = require('./account-manager');

async function run() {
    try {
        const manager = await getUnlockedManager();

        const account = await manager.getAccount('0');

        await account.sync();

        // Get a tokenId from your account balance after running example
        // 22-mint-native-tokens.js
        let tokenId =
            '0x08e6210d29881310db2afde095e594f6f006fcdbd06e7a83b74bd2bdf3b5190d0e0200000000';
        // `100` hex encoded
        let meltAmount = "0xc8"

        const response = await account.decreaseNativeTokenSupply(tokenId, meltAmount);

        console.log(response);

        console.log(
            `Check your block on ${process.env.NODE_URL}/api/core/v2/blocks/${response.blockId}`,
        );
    } catch (error) {
        console.log('Error: ', error);
    }
    process.exit(0);
}

run();
