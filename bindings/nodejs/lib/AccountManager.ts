// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { MessageHandler } from './MessageHandler';
import { Account } from './Account';

import type {
    AccountId,
    Auth,
    EventType,
    AccountManagerOptions,
    CreateAccountPayload,
    NodeInfoWrapper,
    ClientOptions,
    AccountSyncOptions
} from '../types'

export class AccountManager {
    private messageHandler: MessageHandler;

    constructor(options: AccountManagerOptions) {
        this.messageHandler = new MessageHandler(options);
    }
    
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

    async deleteStorage(): Promise<void> {
        await this.messageHandler.sendMessage({
            cmd: 'DeleteStorage',
        })
    }


    destroy() {
        this.messageHandler.destroy();
    }

    async getAccount(accountId: AccountId): Promise<Account> {
        const response = await this.messageHandler.sendMessage({
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

        const accounts: Account[] = [];

        for (const account of payload) {
            accounts.push(new Account(account, this.messageHandler));
        }
        return accounts;
    }

    async getNodeInfo(url?: string, auth?: Auth): Promise<NodeInfoWrapper> {
        return JSON.parse(
            await this.messageHandler.sendMessage({
                cmd: 'GetNodeInfo',
                payload: { url, auth },
            }),
        ).payload;
    }
    
    async setStrongholdPassword(password: string): Promise<void> {
        await this.messageHandler.sendMessage({
            cmd: 'SetStrongholdPassword',
            payload: password,
        });
    }

    async generateMnemonic(): Promise<string> {
        const response = await this.messageHandler.sendMessage({
            cmd: 'GenerateMnemonic',
        });
        return JSON.parse(response).payload;
    }
    
    async storeMnemonic(mnemonic: string): Promise<void> {
        await this.messageHandler.sendMessage({
            cmd: 'StoreMnemonic',
            payload: mnemonic,
        });
    }
    
    async verifyMnemonic(mnemonic: string): Promise<void> {
        await this.messageHandler.sendMessage({
            cmd: 'VerifyMnemonic',
            payload: mnemonic,
        });
    }

    async setClientOptions(options: ClientOptions): Promise<void> {
        await this.messageHandler.sendMessage({
            cmd: 'SetClientOptions',
            payload: options,
        })
    }

    async startBackgroundSync(options?: AccountSyncOptions, interval?: number): Promise<void> {
        await this.messageHandler.sendMessage({
            cmd: 'StartBackgroundSync',
            payload: {
                options,
                interval,
            }
        })
    }

    async stopBackgroundSync(): Promise<void> {
        await this.messageHandler.sendMessage({
            cmd: 'StopBackgroundSync',
        })
    }

    async restoreBackup(source: string, password: string): Promise<void> {
        await this.messageHandler.sendMessage({
            cmd: 'RestoreBackup',
            payload: {
                source,
                password,
            },
        });
    }

    listen(
        eventTypes: EventType[],
        callback: (error: Error, result: string) => void,
    ): void {
        return this.messageHandler.listen(eventTypes, callback);
    }
}
