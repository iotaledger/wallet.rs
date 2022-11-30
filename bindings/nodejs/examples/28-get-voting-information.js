/**
 * This example creates a new voting event
 */
const getUnlockedManager = require('./account-manager');

// Replace with an event id published to the Tangle
const EVENT_ID = '0x7ba318a26a1f639389a3428f159f40aebbcc776a4f8ca17de4fa45221ac79fbd'

async function run() {
    try {
        const manager = await getUnlockedManager();

        // store the event information from a node locally
        await manager.registerParticipationEvent(
            EVENT_ID,
            [{url: 'https://api.testnet.shimmer.network'}]
        )
        
        // get the participation events that are stored in wallet.rs
        const events = await manager.getParticipationEvents()
        console.log('List of registered events:', events)
        
        // get the participation event with a specific id that is stored in wallet.rs
        const event = await manager.getParticipationEvent(EVENT_ID)
        console.log(`Event ${EVENT_ID}:`, event)
        
        // get the status of a participation event that is stored in wallet.rs
        const eventStatus = await manager.getParticipationEventStatus(EVENT_ID)
        console.log('Event Status:', eventStatus)
        
        // removes the participation events from a manager
        await manager.deregisterParticipationEvent(EVENT_ID)

        process.exit(0)
    } catch (error) {
        console.log('Error: ', error);
        process.exit(1);
    }
}

run();
 