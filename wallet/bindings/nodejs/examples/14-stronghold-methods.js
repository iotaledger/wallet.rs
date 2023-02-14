/**
 * This example demonstrates the usage of isStrongholdPasswordAvailable and clearStrongholdPassword
 */
const getUnlockedManager = require('./account-manager');

async function run() {
    try {
        const manager = await getUnlockedManager();

        let isStrongholdUnlocked = await manager.isStrongholdPasswordAvailable();
        console.log('Stronghold unlocked: ', isStrongholdUnlocked)

        await manager.setStrongholdPasswordClearInterval(1000);
        // eslint-disable-next-line no-undef
        setTimeout(
            async () => {
                isStrongholdUnlocked = await manager.isStrongholdPasswordAvailable();
                console.log('Stronghold unlocked: ', isStrongholdUnlocked);
            },
            2000
        )
        isStrongholdUnlocked = await manager.isStrongholdPasswordAvailable();
        console.log('Stronghold locked: ', !isStrongholdUnlocked)
    } catch (error) {
        console.log('Error: ', error);
    }

    setTimeout(
        () => process.exit(0), 3000
    )
}

run()