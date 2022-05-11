// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { MessageHandler } from './MessageHandler';
import { Account } from './Account';

import type {
    AccountId,
    EventType,
    AccountManagerOptions,
    CreateAccountPayload,
    NodeInfo
} from '../types'

export class AccountManager {
    private messageHandler: MessageHandler;

    constructor(options: AccountManagerOptions) {
        this.messageHandler = new MessageHandler(options);
    }

    async getAccount(accountId: AccountId): Promise<Account> {
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

    async getAccounts(): Promise<Account[]> {
        const response = await this.messageHandler.sendMessage({
            cmd: 'GetAccounts',
        });

        const { payload } = JSON.parse(response);

        let accounts: Account[] = [];
        
        for (const account of payload) {
            accounts.push(new Account(account, this.messageHandler));
        }
        return accounts;
    }

    async getNodeInfo(url?: string, auth?: any): Promise<NodeInfo> {
        return JSON.parse(
            await this.messageHandler.sendMessage({
                    cmd: 'GetNodeInfo',
                    payload: {url, auth}
                }
            )
        ).payload;
    }
    
    /**
     * The coin type only needs to be set on the first account
     */
    async createAccount(payload: CreateAccountPayload): Promise<Account> {
        const response = await this.messageHandler
            .sendMessage({
                cmd: 'CreateAccount',
                payload,
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
    async generateMnemonic(): Promise<string> {
        const response = await this.messageHandler.sendMessage({
            cmd: 'GenerateMnemonic',
        });
        return JSON.parse(response).payload
    }
    
    /**
     * TODO: Replace string type with proper type
     */
    async storeMnemonic(mnemonic: string): Promise<string> {
        return this.messageHandler.sendMessage({
            cmd: 'StoreMnemonic',
            payload: mnemonic,
        });
    }
    
    /**
     * TODO: Replace string type with proper type
     */
    async verifyMnemonic(mnemonic: string): Promise<string> {
        return this.messageHandler.sendMessage({
            cmd: 'VerifyMnemonic',
            payload: mnemonic,
        })
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
