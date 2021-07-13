// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

const addon = require('../../index.node');
const utils = require('../utils.js');

let { syncAsync, getNodeInfoAsync, generateAddress, latestAddress, balance, sendAsync, id  } = addon;



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

class Account {
    constructor(accountId, account) {
      console.log("Account constructor called.");
      this.accountId = accountId;
      this.account = account;
    }
    async sync(options) {
        return await syncAsync(JSON.stringify(options), this.account);
    }

    async getNodeInfo(url) {
        return await getNodeInfoAsync(url, this.account);
    }

    generateAddress() {
        return generateAddress();
    }

    async latestAddress() {
        return latestAddress();
    }

    async balance() {
        return balance();
    }

    async send(transfer) {
        return await sendAsync(JSON.stringify(transfer), this.account);
    }

  };

  module.exports.Account = Account;


module.exports.Account = Account;
