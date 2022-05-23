/**
 * This example demonstrates the usage of isStrongholdPasswordAvailable and clearStrongholdPassword
 */
const getUnlockedManager = require('./account-manager');

async function run() {
    try {
        const manager = await getUnlockedManager();

        let isStrongholdUnlocked = await manager.isStrongholdPasswordAvailable();
        console.log('Stronghold unlocked: ', isStrongholdUnlocked)

        await manager.clearStrongholdPassword();

        isStrongholdUnlocked = await manager.isStrongholdPasswordAvailable();
        console.log('Stronghold locked: ', !isStrongholdUnlocked)
    } catch (error) {
        console.log('Error: ' + error);
    }
    process.exit(0);
}

run()