/**
 * This example listen to the NewOutput event
 */

require('dotenv').config();
const { AccountManager } = require('@iota/wallet');

async function run() {
    try {
        const manager = new AccountManager({
            storagePath: './alice-database',
        });

        const callback = function(err, data) {
            if(err) console.log("err:", err)

            const event = JSON.parse(data)
            console.log("Event for account:", event.accountIndex)
            console.log("data:", event.event)

            // Exit after receiving an event
            process.exit(0);
        }

        // provide event type to filter only for events with this type
        manager.listen(['NewOutput'], callback);

        const account = await manager.getAccount('Alice');

        // Use the Faucet to send testnet tokens to your address:
        console.log("Fill your address with the Faucet: https://faucet.testnet.shimmer.network/")
        const addressObjects = await account.addresses();
        console.log('Send funds to:', addressObjects[0].address);

        // Sync every 5 seconds until the faucet transaction gets confirmed
        for (let i = 0; i < 100; i++) {
            await new Promise(resolve => setTimeout(resolve, 5000));
        
            // Sync to detect new outputs
            // syncOnlyMostBasicOutputs if not interested in outputs that are timelocked, 
            // have a storage deposit return or are nft/alias/foundry outputs
            await account.sync({ syncOnlyMostBasicOutputs: true });
        }

    } catch (error) {
        console.log('Error: ', error);
    }
    process.exit(0);
}

run();
