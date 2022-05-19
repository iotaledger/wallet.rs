const getUnlockedManager = require('./account-manager');

async function run() {
    try {
        const manager = await getUnlockedManager();
        const nodeInfo = await manager.getNodeInfo();
        console.log('Node Info:', nodeInfo);
    } catch (error) {
        console.log('Error: ' + error);
    }
    process.exit();
}

run();
