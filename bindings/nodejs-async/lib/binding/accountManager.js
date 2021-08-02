// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

const addon = require('../../index.node');
const utils = require('../utils.js');
const acc = require('./account.js');
const { SyncedAccount } = require('./synced_account.js')

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
    removeAccount,
    syncAccounts,
    setClientOptions,
    internalTransfer,
    isLatestAddressUnused,
    getBalanceChangeEvents,
    getBalanceChangeEventCount,
    getTransactionConfirmationEvents,
    getTransactionConfirmationEventCount,
    getNewTransactionEvents,
    getNewTransactionEventCount,
    getReattachmentEvents,
    getReattachmentEventCount,
    getBroadcastEvents,
    getBroadcastEventCount,
} = addon;

let { Account } = acc;

const syncAccountsAsync = utils.promisify(syncAccounts);
const syncInternalTransfer = utils.promisify(internalTransfer);
const syncIsLatestAddressUnused = utils.promisify(isLatestAddressUnused);
class AccountManager {
    constructor(options) {
        console.log("AccountManager constructor called.");
        console.log(options)
        console.log(JSON.stringify(options));
        this.accountManager = accountManagerNew(JSON.stringify(options));
        console.log(this.accountManager);
    }
    getAccount(accountId) {
        let inner_account = getAccount.apply(this.accountManager, [accountId]);
        return new Account(inner_account);
    }

    getAccounts() {
        let inner_accounts = getAccounts.apply(this.accountManager);
        return inner_accounts.map(a => new Account(a));
    }

    removeAccount(id) {
        return removeAccount.apply(this.accountManager, [id]);
    }

    setClientOptions(options) {
        return setClientOptions.apply(this.accountManager, [options]);
    }

    async syncAccounts() {
        return await syncAccountsAsync.apply(this.accountManager).then(id => new SyncedAccount(id));
    }

    async internalTransfer(fromAccount, toAccount, amount) {
        return await syncInternalTransfer.apply(this.accountManager, [fromAccount, toAccount, amount]);
    }

    async isLatestAddressUnused() {
        return await syncIsLatestAddressUnused.apply(this.accountManager);
    }

    createAccount(account) {
        let acc = createAccount.apply(this.accountManager, [JSON.stringify(account)]);
        // console.log(acc);
        return new Account(acc);
    }

    setStrongholdPassword(password) {
        return setStrongholdPassword.apply(this.accountManager, [password]);
    }

    storeMnemonic(signerType, mnemonic) {
        console.log(signerType, mnemonic);

        return storeMnemonic.apply(this.accountManager, [signerType, mnemonic].filter(e => e != undefined));
    }

    backup(destination, password) {
        return backup.apply(this.accountManager, [destination, password]);
    }

    importAccounts(backupPath, password) {
        return importAccounts.apply(this.accountManager, [backupPath, password]);
    }
    setStoragePassword(password) {
        return setStoragePassword.apply(this.accountManager, [password]);
    }

    changeStrongholdPassword(currentPassword, oldPassword) {
        return changeStrongholdPassword.apply(this.accountManager, [currentPassword, oldPassword]);
    }

    generateMnemonic() {
        return generateMnemonic.apply(this.accountManager);
    }

    getBalanceChangeEvents(count, skip, fromTimestamp) {
        return getBalanceChangeEvents.apply(this.accountManager, [count, skip, fromTimestamp].filter(e => e != undefined));
    }

    getBalanceChangeEventCount(fromTimestamp) {
        return getBalanceChangeEventCount.apply(this.accountManager, [fromTimestamp].filter(e => e != undefined));
    }

    getTransactionConfirmationEvents(count, skip, fromTimestamp) {
        return getTransactionConfirmationEvents.apply(this.accountManager, [count, skip, fromTimestamp].filter(e => e != undefined));
    }

    getTransactionConfirmationEventCount(fromTimestamp) {
        return getTransactionConfirmationEventCount.apply(this.accountManager, [fromTimestamp].filter(e => e != undefined));
    }

    getNewTransactionEvents(count, skip, fromTimestamp) {
        return getNewTransactionEvents.apply(this.accountManager, [count, skip, fromTimestamp].filter(e => e != undefined));
    }

    getNewTransactionEventCount(fromTimestamp) {
        return getNewTransactionEventCount.apply(this.accountManager, [fromTimestamp].filter(e => e != undefined));
    }

    getReattachmentEvents(count, skip, fromTimestamp) {
        return getReattachmentEvents.apply(this.accountManager, [count, skip, fromTimestamp].filter(e => e != undefined));
    }

    getReattachmentEventCount(fromTimestamp) {
        return getReattachmentEventCount.apply(this.accountManager, [fromTimestamp].filter(e => e != undefined));
    }

    getBroadcastEvents(count, skip, fromTimestamp) {
        return getBroadcastEvents.apply(this.accountManager, [count, skip, fromTimestamp].filter(e => e != undefined));
    }

    getBroadcastEventCount(fromTimestamp) {
        return getBroadcastEventCount.apply(this.accountManager, [fromTimestamp].filter(e => e != undefined));
    }
};

module.exports.AccountManager = AccountManager;