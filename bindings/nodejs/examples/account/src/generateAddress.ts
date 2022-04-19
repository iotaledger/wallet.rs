/**
 * This example demonstrates how to generate an address
 */

import { Account, Address, AccountManager } from '@iota/wallet';

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
    const addresses: Address[] = await account.generateAddresses();

    console.info('Generated addresses: ', addresses);
}

run();
