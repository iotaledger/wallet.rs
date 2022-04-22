const { AccountManager } = require('@iota/wallet');

const accountManagerOptions = {
    storagePath: './alice-database',
    clientOptions: {
        nodes: [
            {
                url: 'https://api.firefly.h.chrysalis-devnet.iota.cafe:14265/',
                auth: null,
                disabled: false,
            },
        ],
        localPow: true,
    },
    signer: {
        Mnemonic:
            'acoustic trophy damage hint search taste love bicycle foster cradle brown govern endless depend situate athlete pudding blame question genius transfer van random vast',
    },
};

const manager = new AccountManager(accountManagerOptions);

module.exports = manager;
