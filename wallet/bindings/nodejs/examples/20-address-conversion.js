/**
 * This example converts a bech32 encoded address to hex and back to bech32.
 */
const getUnlockedManager = require('./account-manager');

async function run() {
    try {
        const manager = await getUnlockedManager();

        const account = await manager.getAccount('0');

        // Generate an address to convert
        const address = await account.generateAddress();
        console.log('New address:', address);

        // Convert the bech32 encoded address to hex
        const hex = await manager.bech32ToHex(address.address);
        console.log('Hex encoded address: ', hex);

        // Convert the hex encoded address back to bech32
        const bech32 = await manager.hexToBech32(hex);
        console.log('Bech32 encoded address: ', bech32);
    } catch (error) {
        console.log('Error: ', error);
    }
    process.exit(0);
}

run();
