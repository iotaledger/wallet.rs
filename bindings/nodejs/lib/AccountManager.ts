// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { MessageHandler } from './MessageHandler';
import { Account } from './Account';

import type {
    EventType,
    AccountManagerOptions,
    CreateAccountPayload
} from '../types'

export class AccountManager {
    private messageHandler: MessageHandler;

    constructor(options: AccountManagerOptions) {
        this.messageHandler = new MessageHandler(options);
    }

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
     */
    async getAccounts(): Promise<any> {
        return this.messageHandler.sendMessage({
            cmd: 'GetAccounts',
        });
    }

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
     */
    async setStrongholdPassword(password: string): Promise<string> {
        return this.messageHandler.sendMessage({
            cmd: 'SetStrongholdPassword',
            payload: password,
        });
    }

    /**
     * TODO: Replace string type with proper type
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

    listen(eventTypes: EventType[], callback: (error: Error, result: string) => void): void {
        return this.messageHandler.listen(eventTypes, callback);
    }
}
