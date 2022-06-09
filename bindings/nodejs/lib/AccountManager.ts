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
    AccountSyncOptions,
    WalletEvent,
} from '../types';

export class AccountManager {
    private messageHandler: MessageHandler;

    constructor(options: AccountManagerOptions) {
        this.messageHandler = new MessageHandler(options);
    }

    async backup(destination: string, password: string): Promise<void> {
        await this.messageHandler.sendMessage({
            cmd: 'Backup',
            payload: {
                destination,
                password,
            },
        });
    }

    async changeStrongholdPassword(password: string): Promise<void> {
        await this.messageHandler.sendMessage({
            cmd: 'ChangeStrongholdPassword',
            payload: {
                password,

            }
        })
    }

    async clearStrongholdPassword(): Promise<void> {
        await this.messageHandler.sendMessage({
            cmd: 'ClearStrongholdPassword',
        });
    }

    /**
     * The coin type only needs to be set on the first account
     */
    async createAccount(payload: CreateAccountPayload): Promise<Account> {
        const response = await this.messageHandler.sendMessage({
            cmd: 'CreateAccount',
            payload,
        });
        return new Account(JSON.parse(response).payload, this.messageHandler);
    }

    async deleteStorage(): Promise<void> {
        await this.messageHandler.sendMessage({
            cmd: 'DeleteStorage',
        });
    }

    destroy(): void {
        this.messageHandler.destroy();
    }

    // TODO: test this
    async emitTestEvent(event: WalletEvent): Promise<void> {
        await this.messageHandler.sendMessage({
            cmd: 'EmitTestEvent',
            payload: event,
        });
    }

    async generateMnemonic(): Promise<string> {
        const response = await this.messageHandler.sendMessage({
            cmd: 'GenerateMnemonic',
        });
        return JSON.parse(response).payload;
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
        const response = await this.messageHandler.sendMessage({
            cmd: 'GetNodeInfo',
            payload: { url, auth },
        });
        return JSON.parse(response).payload;
    }

    async isStrongholdPasswordAvailable(): Promise<boolean> {
        const response = await this.messageHandler.sendMessage({
            cmd: 'IsStrongholdPasswordAvailable',
        });
        return JSON.parse(response).payload;
    }

    listen(
        eventTypes: EventType[],
        callback: (error: Error, result: string) => void,
    ): void {
        return this.messageHandler.listen(eventTypes, callback);
    }

    // TODO: test this
    async recoverAccounts(
        accountGapLimit: number,
        addressGapLimit: number,
    ): Promise<Account[]> {
        const response = await this.messageHandler.sendMessage({
            cmd: 'RecoverAccounts',
            payload: {
                accountGapLimit,
                addressGapLimit,
            },
        });
        return JSON.parse(response).payload;
    }

    async removeLatestAccount(): Promise<void> {
        await this.messageHandler.sendMessage({
            cmd: 'RemoveLatestAccount',
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

    async setClientOptions(options: ClientOptions): Promise<void> {
        await this.messageHandler.sendMessage({
            cmd: 'SetClientOptions',
            payload: options,
        });
    }

    async setStrongholdPassword(password: string): Promise<void> {
        await this.messageHandler.sendMessage({
            cmd: 'SetStrongholdPassword',
            payload: password,
        });
    }

    async setStrongholdPasswordClearInterval(
        intervalInMilliseconds?: number,
    ): Promise<void> {
        await this.messageHandler.sendMessage({
            cmd: 'SetStrongholdPasswordClearInterval',
            payload: intervalInMilliseconds,
        });
    }

    async startBackgroundSync(
        options?: AccountSyncOptions,
        intervalInMilliseconds?: number,
    ): Promise<void> {
        await this.messageHandler.sendMessage({
            cmd: 'StartBackgroundSync',
            payload: {
                options,
                intervalInMilliseconds,
            },
        });
    }

    async stopBackgroundSync(): Promise<void> {
        await this.messageHandler.sendMessage({
            cmd: 'StopBackgroundSync',
        });
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
}
