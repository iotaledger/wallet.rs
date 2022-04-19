/**
 * This example demonstrates how to check account balance
 */

import { Account, AccountManager, AccountBalance } from '@iota/wallet';

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

    const account: Account = await manager.getAccount(0);

    // Sync account with the network before getting the balance
    await account.sync();

    const balance: AccountBalance = await account.balance();

    console.info('Account balance: ', balance);
}

run();
