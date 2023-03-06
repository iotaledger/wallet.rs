/**
 * This example changes the stronghold password.
 */
const path = require('path')
require('dotenv').config({ path: path.resolve(__dirname, '.env') });
const getUnlockedManager = require('./account-manager');

const NEW_PASSWORD = 'new_super_secure_password'

async function run() {
    try {
        const manager = await getUnlockedManager();

        await manager.changeStrongholdPassword(process.env.SH_PASSWORD, NEW_PASSWORD)

        // Clear the password from memory
        await manager.clearStrongholdPassword()
    
        // Set the new password to see if it works
        await manager.setStrongholdPassword(NEW_PASSWORD)

        // Reverts to original password
        await manager.changeStrongholdPassword(NEW_PASSWORD, process.env.SH_PASSWORD)
    } catch (error) {
        console.log('Error: ', error);
    }
    process.exit(0);
}

run();
