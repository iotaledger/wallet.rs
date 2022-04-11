// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import MessageHandler from './MessageHandler';
import Account from './Account';

import type {
    EventType,
    AccountManagerOptions,
    CreateAccountPayload
} from './types'

/**
 * Account Manager class
 */
export default class AccountManager {
    messageHandler: MessageHandler;

    /**
     * Creates a new instance of Account Manager
     * 
     * @param {AccountManagerOptions} options 
     */
    constructor(options: AccountManagerOptions) {
        this.messageHandler = new MessageHandler(options);
    }

    /**
     * Get account object for provided id
     * 
     * @param {string} accountId 
     * 
     * @returns {Promise<Account>}
     */
    async getAccount(accountId: string): Promise<Account> {
        const response = await this.messageHandler
            .sendMessage({
                cmd: 'GetAccount',
                payload: accountId,
            });

        const account = new Account(
            JSON.parse(response).payload,
            this.messageHandler,
        );

        return account;
    }

    /**
     * TODO: Replace any with proper type
     * 
     * Gets account associated with manager
     * 
     * @returns {Promise<any>}
     */
    async getAccounts(): Promise<any> {
        return this.messageHandler.sendMessage({
            cmd: 'GetAccounts',
        });
    }

    /**
     * Creates a new account
     * 
     * @param {Account} account
     *  
     * @returns {Promise<Account>}
     */
    async createAccount(account: CreateAccountPayload): Promise<Account> {
        const response = await this.messageHandler
            .sendMessage({
                cmd: 'CreateAccount',
                payload: account,
            });

        return new Account(
            JSON.parse(response).payload,
            this.messageHandler,
        );
    }

    /**
     * TODO: Replace string type with proper type
     * 
     * Sets stronghold password
     * 
     * @param {string} password
     *  
     * @returns {Promise<string>} 
     */
    async setStrongholdPassword(password: string): Promise<string> {
        return this.messageHandler.sendMessage({
            cmd: 'SetStrongholdPassword',
            payload: password,
        });
    }

    /**
     * TODO: Replace string type with proper type
     * 
     * Stores mnemonic
     * 
     * @param {string} mnemonic
     *  
     * @returns {Promise<string>} 
     */
    async storeMnemonic(mnemonic: string): Promise<string> {
        return this.messageHandler.sendMessage({
            cmd: 'StoreMnemonic',
            payload: {
                signerType: {
                    type: 'Stronghold',
                },
                mnemonic,
            },
        });
    }

    /**
     * TODO: Replace string type with proper type
     * 
     * Backs up stronghold file
     * 
     * @param {string} destination
     * @param {string} password
     *  
     * @returns {Promise<string>} 
     */
    async backup(destination: string, password: string): Promise<string> {
        return this.messageHandler.sendMessage({
            cmd: 'Backup',
            payload: {
                destination,
                password,
            },
        });
    }

    /**
     * TODO: Replace string type with proper type
     * 
     * Imports from stronghold file
     * 
     * @param {string} backupPath
     * @param {string} password
     *  
     * @returns {Promise<string>} 
     */
    async importAccounts(backupPath: string, password: string): Promise<string> {
        return this.messageHandler.sendMessage({
            cmd: 'RestoreBackup',
            payload: {
                backupPath,
                password,
            },
        });
    }

    /**
     * Listen to events
     * 
     * @param {EventType[]} eventTypes 
     * @param {Function} callback 
     * 
     * @returns {void}
     */
    listen(eventTypes: EventType[], callback: (error: Error, result: string) => void): void {
        return this.messageHandler.listen(eventTypes, callback);
    }
}
