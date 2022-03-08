// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

const addon = require('../../build/Release/index.node');
const utils = require('../utils.js');
const { SyncedAccount } = require('./syncedAccount.js');

const {
    sync,
    getNodeInfo,
    generateAddress,
    latestAddress,
    balance,
    send,
    id,
    alias,
    index,
    listMessages,
    listAddresses,
    setAlias,
    setClientOptions,
    getMessage,
    getAddress,
    getUnusedAddress,
    generateAddresses,
    isLatestAddressUnused,
    consolidateOutputs,
    repost,
    sendToMany,
    messageCount,
} = addon;

const syncAsync = utils.promisify(sync);
const sendAsync = utils.promisify(send);
const sendToManyAsync = utils.promisify(sendToMany);
const getNodeInfoAsync = utils.promisify(getNodeInfo);
const isLatestAddressUnusedAsync = utils.promisify(isLatestAddressUnused);
const consolidateOutputsAsync = utils.promisify(consolidateOutputs);
const repostAsync = utils.promisify(repost);

class Account {
    constructor(account) {
        this.account = account;
    }

    id() {
        return id.apply(this.account);
    }

    alias() {
        return alias.apply(this.account);
    }

    index() {
        return index.apply(this.account);
    }

    generateAddress() {
        return JSON.parse(generateAddress.apply(this.account));
    }

    generateAddresses(amount) {
        return generateAddresses.apply(this.account, [amount]).map((address) => JSON.parse(address));
    }

    latestAddress() {
        return JSON.parse(latestAddress.apply(this.account));
    }

    listMessages(count, from, messageType) {
        return listMessages
            .apply(
                this.account,
                [count, from, messageType].filter((e) => e != undefined),
            )
            .map((msg) => JSON.parse(msg));
    }

    messageCount(messageType) {
        return messageCount.apply(this.account, [messageType]);
    }

    listAddresses(unspent) {
        return listAddresses
            .apply(
                this.account,
                [unspent].filter((e) => e != undefined),
            )
            .map((address) => JSON.parse(address));
    }

    setAlias(alias) {
        return setAlias.apply(this.account, [alias]);
    }

    setClientOptions(options) {
        return setClientOptions.apply(this.account, [JSON.stringify(options)]);
    }

    getMessage(id) {
        return JSON.parse(getMessage.apply(this.account, [id]));
    }

    getAddress(addressBech32) {
        return getAddress.apply(this.account, [addressBech32]);
    }

    getUnusedAddress() {
        return getUnusedAddress.apply(this.account);
    }

    balance() {
        return JSON.parse(balance.apply(this.account));
    }

    async sync(options) {
        return await syncAsync
            .apply(
                this.account,
                [JSON.stringify(options)].filter((e) => e != undefined),
            )
            .then((id) => new SyncedAccount(id));
    }

    async getNodeInfo(url, auth) {
        return await getNodeInfoAsync
            .apply(
                this.account,
                [url, JSON.stringify(auth)].filter((e) => e != undefined),
            )
            .then((nodeInfo) => JSON.parse(nodeInfo));
    }

    async send(address, amount, options) {
        return await sendAsync
            .apply(
                this.account,
                [address, amount, JSON.stringify(options)].filter(
                    (e) => e != undefined,
                ),
            )
            .then((message) => JSON.parse(message));
    }

    async sendToMany(outputs, options) {
        return await sendToManyAsync
            .apply(
                this.account,
                [outputs, JSON.stringify(options)].filter(
                    (e) => e != undefined,
                ),
            )
            .then((message) => JSON.parse(message));
    }

    async isLatestAddressUnused() {
        return await isLatestAddressUnusedAsync.apply(this.account);
    }

    async consolidateOutputs(includeDustAllowanceOutput) {
        return await consolidateOutputsAsync
            .apply(this.account, [includeDustAllowanceOutput])
            .map((message) => JSON.parse(message));
    }

    async retry(messageId) {
        return await repostAsync.apply(this.account, [messageId, 'Retry']);
    }

    async reattach(messageId) {
        return await repostAsync.apply(this.account, [messageId, 'Reattach']);
    }

    async promote(messageId) {
        return await repostAsync.apply(this.account, [messageId, 'Promote']);
    }
}

module.exports.Account = Account;
