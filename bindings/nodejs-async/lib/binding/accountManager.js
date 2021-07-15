// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

const addon = require('../../index.node');
const utils = require('../utils.js');
const acc = require('./account.js');

let {
    accountManagerNew,
    getAccount,
    getAccounts,
    createAccount,
    setStrongholdPassword,
    storeMnemonic,
    backup,
    importAccounts,
    setStoragePassword,
    changeStrongholdPassword,
    generateMnemonic,
    id
} = addon;
let { Account } = acc;
class AccountManager {
    constructor(options) {
        console.log("AccountManager constructor called.");
        console.log(options)
        console.log(JSON.stringify(options));
        this.accountManager = accountManagerNew(JSON.stringify(options));
        console.log(this.accountManager);
    }
    getAccount(accountId) {
        let inner_account = getAccount(accountId, this.accountManager);
        return new Account(inner_account);
    }

    getAccounts() {
        let inner_accounts = getAccounts(this.accountManager);
        let result = [];
        for (acc in inner_accounts) {
            result.push(new Account(acc));
        }
        return result;
    }

    createAccount(account) {
        let acc = createAccount(JSON.stringify(account), this.accountManager);
        // console.log(acc);
        return new Account(acc);
    }

    setStrongholdPassword(password) {
        return setStrongholdPassword(password, this.accountManager);
    }

    storeMnemonic(signerType, mnemonic) {
        console.log(signerType, mnemonic);
        if (mnemonic == undefined) {
            return storeMnemonic(signerType, this.accountManager);
        }
        return storeMnemonic(signerType, mnemonic, this.accountManager);
    }

    backup(destination, password) {
        return backup(destination, password, this.accountManager);
    }

    importAccounts(backupPath, password) {
        return importAccounts(backupPath, password, this.accountManager);
    }
    setStoragePassword(password) {
        return setStoragePassword(password, this.accountManager);
    }

    changeStrongholdPassword(currentPassword, oldPassword) {
        return changeStrongholdPassword(currentPassword, oldPassword, this.accountManager);
    }

    generateMnemonic() {
        return generateMnemonic(this.accountManager);
    }
};

module.exports.AccountManager = AccountManager;