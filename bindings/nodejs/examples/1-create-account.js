/**
 * This example creates a new database and account
 */

require('dotenv').config();

async function run() {
    const { AccountManager, initLogger } = require('@iota/wallet');

    initLogger({
        color_enabled: true,
        outputs: [{
            name: './wallet.log',
            level_filter: 'debug'
        }]
    })
    
    const manager = new AccountManager({
        storagePath: './alice-database',
        clientOptions: {
            "nodes":[
               {
                  "url":"http://localhost:14265/",
                  "auth":null,
                  "disabled":false
               }
            ],
            "localPow":true,
         },
        signer:{
            "Mnemonic":"acoustic trophy damage hint search taste love bicycle foster cradle brown govern endless depend situate athlete pudding blame question genius transfer van random vast"
        }
    });

    try {
        // await manager.setStrongholdPassword(process.env.SH_PASSWORD);
        // await manager.storeMnemonic();

        const account = await manager.createAccount({
            alias: 'Alice',
        });
        console.log('Account created:', account);
    } catch (error) {
        console.log('Error: ' + error);
    }
}

run();
