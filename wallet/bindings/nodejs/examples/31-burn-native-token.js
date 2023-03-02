/**
 * This example will burn native tokens
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
            '0x08e81324605a946192ad414cf10da992ba5b97001ed4242de084e72cf19a924f7b0100000000';
        // `100` hex encoded
        let burnAmount = "0x64"

        const response = await account.burnNativeToken(tokenId, burnAmount);

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
