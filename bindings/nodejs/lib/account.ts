// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import type MessageHandler from './MessageHandler';
import type {
    AccountBalance,
    Address,
    AccountSyncOptions,
    AccountMeta,
    NodeInfo,
    Transfer,
    ClientOptions
} from './types';

/**
 * Account class
 */
export default class Account {
    meta: AccountMeta;
    messageHandler: MessageHandler

    /**
    * Creates a new instance of Account
    * 
    * @param {AccountMeta} accountMeta
    * @param {MessageHandler} messageHandler 
    */
    constructor(
        accountMeta: AccountMeta,
        messageHandler: MessageHandler
    ) {
        this.meta = accountMeta;
        this.messageHandler = messageHandler;
    }

    /**
     * Returns account alias
     * 
     * @returns {string}
     */
    alias(): string {
        return this.meta.alias;
    }

    /**
     * Sync an account
     * 
     * @param {AccountSyncOptions} options
     * 
     * @returns {Promise<void>} 
     */
    async sync(options: AccountSyncOptions): Promise<void> {
        await this.messageHandler.sendMessage({
            cmd: 'CallAccountMethod',
            payload: {
                // TODO: Change to camelCase
                account_id: this.meta.index,
                method: {
                    name: 'SyncAccount',
                    data: options || {},
                },
            },
        })
    }

    /**
     * TODO: Test this method through example and see if the interface is correct
     * 
     * Gets node info for the node set against this account
     * 
     * @param {string} url
     *  
     * @returns {Promise<NodeInfo>}
     */
    async getNodeInfo(url: string): Promise<NodeInfo> {
        return JSON.parse(
            await this.messageHandler.sendMessage({
                cmd: 'CallAccountMethod',
                payload: {
                    account_id: this.meta.index,
                    method: {
                        name: 'GetNodeInfo',
                        data: [url],
                    },
                },
            }),
        );
    }

    /**
     * Generates addresses for the account
     * 
     * @returns {Promise<Address[]>}
     */
    async generateAddresses(): Promise<Address[]> {
        const response = await this.messageHandler.sendMessage({
            cmd: 'CallAccountMethod',
            payload: {
                account_id: this.meta.index,
                method: {
                    name: 'GenerateAddresses',
                    data: {
                        // TODO: Why is the amount set to 1 here?
                        amount: 1,
                    }
                },
            },
        });

        return JSON.parse(response).payload;
    }

    /**
     * Returns the latest address for the account
     * 
     * @returns {Promise<Address>}
     */
    async latestAddress(): Promise<Address> {
        const response = await this.messageHandler.sendMessage({
            cmd: 'CallAccountMethod',
            payload: {
                account_id: this.meta.index,
                method: {
                    name: 'GetLatestAddress',
                },
            },
        });

        return JSON.parse(response).payload;
    }

    /**
     * Returns account balance
     * 
     * @returns {Promise<AccountBalance>}
     */
    async balance(): Promise<AccountBalance> {
        const response = await this.messageHandler.sendMessage({
            cmd: 'CallAccountMethod',
            payload: {
                account_id: this.meta.index,
                method: {
                    name: 'GetBalance',
                },
            },
        });

        return JSON.parse(response).payload;
    }

    /**
     * TODO: Replace any with sent message
     * 
     * Make a transaction
     * 
     * @param {Transfer} transfer 
     * 
     * @returns {Promise<any>}
     */
    async send(transfer: Transfer): Promise<any> {
        const response = await this.messageHandler.sendMessage({
            cmd: 'SendTransfer',
            payload: {
                account_id: this.meta.index,
                transfer,
            },
        });

        return JSON.parse(response).payload;
    }

    /**
     * TODO: Replace any with proper response type
     * 
     * @param {ClientOptions} options 
     * 
     * @returns {Promise<any>}
     */
    async setClientOptions(options: ClientOptions): Promise<any> {
        const response = await this.messageHandler.sendMessage({
            cmd: 'SetClientOptions',
            payload: options,
        });

        return JSON.parse(response).payload;
    }
}
