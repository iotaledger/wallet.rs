// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

class AccountForMessages {
    constructor(accountData, messageHandler) {
        this.accountData = accountData;
        this.messageHandler = messageHandler;
    }

    alias() {
        return this.accountData.alias;
    }

    async sync(options) {
        return JSON.parse(
            await this.messageHandler.sendMessage({
                cmd: 'CallAccountMethod',
                payload: {
                    accountId: this.accountData.id,
                    method: {
                        name: 'SyncAccount',
                        data: options || {},
                    },
                },
            }),
        );
    }

    async getNodeInfo(url) {
        return JSON.parse(
            await this.messageHandler.sendMessage({
                cmd: 'CallAccountMethod',
                payload: {
                    accountId: this.accountData.id,
                    method: {
                        name: 'GetNodeInfo',
                        data: [url],
                    },
                },
            }),
        );
    }

    async generateAddress() {
        return JSON.parse(
            await this.messageHandler.sendMessage({
                cmd: 'CallAccountMethod',
                payload: {
                    accountId: this.accountData.id,
                    method: {
                        name: 'GenerateAddress',
                    },
                },
            }),
        );
    }

    async latestAddress() {
        return JSON.parse(
            await this.messageHandler.sendMessage({
                cmd: 'CallAccountMethod',
                payload: {
                    accountId: this.accountData.id,
                    method: {
                        name: 'GetLatestAddress',
                    },
                },
            }),
        );
    }

    async balance() {
        return JSON.parse(
            await this.messageHandler.sendMessage({
                cmd: 'CallAccountMethod',
                payload: {
                    accountId: this.accountData.id,
                    method: {
                        name: 'GetBalance',
                    },
                },
            }),
        );
    }

    async send(transfer) {
        return JSON.parse(
            await this.messageHandler.sendMessage({
                cmd: 'SendTransfer',
                payload: {
                    accountId: this.accountData.id,
                    transfer,
                },
            }),
        );
    }

    async setClientOptions(options) {
        return JSON.parse(
            await this.messageHandler.sendMessage({
                cmd: 'SetClientOptions',
                payload: options,
            }),
        );
    }
}

module.exports.AccountForMessages = AccountForMessages;
