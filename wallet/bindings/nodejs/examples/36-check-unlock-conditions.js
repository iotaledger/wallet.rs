/**
 * In this example we check if an output has only an address unlock condition and that the address is from the account.
 */

const getUnlockedManager = require('./account-manager');

async function run() {
    try {
        let manager = await getUnlockedManager();

        const account = await manager.getAccount('Alice');

        let accountAddresses = await account.addresses()

        const output = await account.prepareOutput({
            recipientAddress: accountAddresses[0].address,
            amount: "1000000",
        });

        let hexEncodedAccountAddresses = await Promise.all(accountAddresses.map(async (a) => await manager.bech32ToHex(a.address)));

        let controlledByAccount = false
        if (output.unlockConditions.length === 1 &&
            output.unlockConditions[0].type === 0 &&
            hexEncodedAccountAddresses.includes(output.unlockConditions[0].address.pubKeyHash)) {
                controlledByAccount = true
        }

        console.log("The output has only an address unlock condition and the address is from the account: " + controlledByAccount)

    } catch (error) {
        console.log('Error: ', error);
    }
    process.exit(0);
}

run();
