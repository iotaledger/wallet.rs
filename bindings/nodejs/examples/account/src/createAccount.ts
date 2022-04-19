/**
 * This example creates a new database and account
 */

import { Account, AccountManager } from '@iota/wallet';

async function run() {
    const manager = new AccountManager({
        storagePath: './__storage__',
        clientOptions: {
            nodes: [
                {
                    url: 'https://api.alphanet.iotaledger.net'
                }
            ],
            localPow: true,
        },
        signer: {
            Mnemonic: 'flight about meadow begin pigeon assault cricket when curve regular degree board river garlic pride salmon online course congress cup tiny south slender carpet'
        }
    });

    const account: Account = await manager.createAccount({ alias: 'Account # 01 ' })

    // Sync account with the network to fetch the latest information
    await account.sync();
    
    console.info('New account created: ', account);
}

run();
