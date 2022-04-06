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
            account_id: this.accountData.index,
            method: {
                name: 'SyncAccount',
                data: options || {},
            },
        })
        return JSON.parse(
            await this.messageHandler.sendMessage({
                cmd: 'CallAccountMethod',
                payload: {
                    account_id: this.accountData.index,
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
                    account_id: this.accountData.index,
                    method: {
                        name: 'GetNodeInfo',
                        data: [url],
                    },
                },
            }),
        );
    }

    async generateAddresses() {
        return JSON.parse(
            await this.messageHandler.sendMessage({
                cmd: 'CallAccountMethod',
                payload: {
                    account_id: this.accountData.index,
                    method: {
                        name: 'GenerateAddresses',
                        data: {
                            amount: 1,
                            // options: {
                            //     internal: false, metadata: {
                            //         syncing: false,
                            //         network: "Network::Testnet",
                            //     }
                            // }
                        }
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
                    account_id: this.accountData.index,
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
                    account_id: this.accountData.index,
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
                    account_id: this.accountData.index,
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

