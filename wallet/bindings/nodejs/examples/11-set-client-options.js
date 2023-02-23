/**
 * This example demonstrates how to update client options.
 */
const getUnlockedManager = require('./account-manager');

async function run() {
    try {
        const manager = await getUnlockedManager();
        const clientOptions = {
            primaryNode: {
                url: 'https://localhost:14265/',
                auth: null,
                disabled: false,
            },
        };
        await manager.setClientOptions(clientOptions);
        const resp = await manager.getNodeInfo();
        console.log(resp);
    } catch (error) {
        console.log('Error: ', error);
    }
    process.exit(0);
}

run();
