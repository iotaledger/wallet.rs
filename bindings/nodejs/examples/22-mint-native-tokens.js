/**
 * This example mints native tokens
 */
const getUnlockedManager = require('./account-manager');

async function run() {
    try {
        const manager = await getUnlockedManager();

        const account = await manager.getAccount('0');

        await account.sync();
        
        // If we omit the AccountAddress field the first address of the account is used by default
        const nativeTokenOptions = {
            // Hello in bytes
            foundryMetadata: '48656c6c6f',
            circulatingSupply: '0x64',
            maximumSupply: '0x64',
        };

        let { transaction } = await account.mintNativeToken(
            nativeTokenOptions,
        );
        console.log('Transaction ID: ', transaction.transactionId);
    } catch (error) {
        console.log('Error: ', error);
    }
    process.exit(0);
}

run();
