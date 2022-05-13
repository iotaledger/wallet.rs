/**
 * This example creates a new database and account
 */

require('dotenv').config();
const manager = require('./account-manager');

async function run() {
    try {
        const clientOptions = {
            nodes: [
                {
                    url: 'https://api.alphanet.iotaledger.net/',
                    auth: null,
                    disabled: false,
                },
            ],
        };
        await manager.setClientOptions(clientOptions);
        const resp = await manager.getNodeInfo();
        console.log(resp);
    } catch (error) {
        console.log('Error: ' + error);
    }
    process.exit(0);
}

run();
