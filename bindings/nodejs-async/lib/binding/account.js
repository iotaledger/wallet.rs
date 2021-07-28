// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

const addon = require('../../index.node');
const utils = require('../utils.js');
const { SyncedAccount } = require('./synced_account.js');

let { sync, getNodeInfo, generateAddress, latestAddress, balance, send, id, listMessages, listAddresses } = addon;



/*
    id(): string;
    index(): number;
    alias(): string;
    balance(): AccountBalance;
    getNodeInfo(url?: string, auth?: Auth): Promise<NodeInfoWrapper>;
    messageCount(messageType?: MessageType): number;
    listMessages(count?: number, from?: number, messageType?: MessageType): Message[]
    listAddresses(unspent?: boolean): Address[]
    sync(options?: SyncOptions): Promise<SyncedAccount>
    send(address: string, amount: number, options?: TransferOptions): Promise<Message>
    sendToMany(outputs: TransferOutput[], options?: TransferOptions): Promise<Message>
    retry(messageId: string): Promise<Message>
    reattach(messageId: string): Promise<Message>
    promote(messageId: string): Promise<Message>
    consolidateOutputs(): Promise<Message[]>
    setAlias(alias: string): void
    setClientOptions(options: ClientOptions): void
    getMessage(id: string): Message | undefined
    getAddress(addressBech32: string): Address | undefined
    generateAddress(): Address
    generateAddresses(amount: number): Address[]
    latestAddress(): Address
    getUnusedAddress(): Address
    isLatestAddressUnused(): Promise<boolean>
*/

const syncAsync = utils.promisify(sync);
const sendAsync = utils.promisify(send);
const getNodeInfoAsync = utils.promisify(getNodeInfo);

class Account {
    constructor(account) {
        console.log("Account constructor called.");
        //   this.accountId = accountId;
        this.account = account;
    }
    async sync(options) {
        return await syncAsync.apply(this.account, [JSON.stringify(options)].filter(e => e != undefined)).then(id => new SyncedAccount(id));
    }

    async getNodeInfo(url, auth) {
        return await getNodeInfoAsync.apply(this.account, [url, JSON.stringify(auth)].filter(e => e != undefined))
    }

    id() {
        return id.apply(this.account);
    }

    generateAddress() {
        return generateAddress.apply(this.account);
    }

    latestAddress() {
        return latestAddress.apply(this.account);
    }

    listMessages(count, from, messageType) {
        return listMessages.apply(this.account, [count, from, messageType].filter(e => e != undefined));
    }

    listAddresses(unspent) {
        return listAddresses.apply(this.account, [unspent].filter(e => e != undefined));
    }

    balance() {
        return balance.apply(this.account);
    }

    async send(transfer) {
        return await sendAsync.apply(this.account, [JSON.stringify(transfer)].filter(e => e != undefined));
    }

};

module.exports.Account = Account;
