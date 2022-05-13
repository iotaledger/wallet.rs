require('dotenv').config();
const manager = require('./account-manager');

async function run() {
    try {
        await manager.deleteStorage();
    } catch (error) {
        console.log('Error: ' + error);
    }
    process.exit();
}

run();
