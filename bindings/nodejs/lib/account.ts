// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import type MessageHandler from './messageHandler';


export default class AccountForMessages {
    accountData: any;
    messageHandler: MessageHandler

    constructor(accountData: any, messageHandler: MessageHandler) {
        this.accountData = accountData;
        this.messageHandler = messageHandler;
    }

    alias() {
        return this.accountData.alias;
    }

    async sync(options: any) {
        console.log({
            account_id: this.accountData.id,
            method: {
                name: 'SyncAccount',
                data: options || {},
            },
        })
        return JSON.parse(
            await this.messageHandler.sendMessage({
                cmd: 'CallAccountMethod',
                payload: {
                    account_id: this.accountData.id,
                    method: {
                        name: 'SyncAccount',
                        data: options || {},
                    },
                },
            }),
        );
    }

    async getNodeInfo(url: string) {
        return JSON.parse(
            await this.messageHandler.sendMessage({
                cmd: 'CallAccountMethod',
                payload: {
                    account_id: this.accountData.id,
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
                    account_id: this.accountData.id,
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
                    account_id: this.accountData.id,
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
                    account_id: this.accountData.id,
                    method: {
                        name: 'GetBalance',
                    },
                },
            }),
        );
    }

    async send(transfer: any) {
        return JSON.parse(
            await this.messageHandler.sendMessage({
                cmd: 'SendTransfer',
                payload: {
                    account_id: this.accountData.id,
                    transfer,
                },
            }),
        );
    }

    async setClientOptions(options: any) {
        return JSON.parse(
            await this.messageHandler.sendMessage({
                cmd: 'SetClientOptions',
                payload: options,
            }),
        );
    }
}

