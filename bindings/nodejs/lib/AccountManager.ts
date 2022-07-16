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
    LedgerStatus,
} from '../types';

/** The AccountManager class. */
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

    /**
     * Transform a bech32 encoded address to a hex encoded address
     */
    async bech32ToHex(bech32Address: string): Promise<string> {
        const response = await this.messageHandler.sendMessage({
            cmd: 'Bech32ToHex',
            payload: bech32Address,
        });
        return JSON.parse(response).payload;
    }

    async changeStrongholdPassword(
        currentPassword: string,
        newPassword: string,
    ): Promise<void> {
        await this.messageHandler.sendMessage({
            cmd: 'ChangeStrongholdPassword',
            payload: {
                currentPassword,
                newPassword,
            },
        });
    }

    async clearStrongholdPassword(): Promise<void> {
        await this.messageHandler.sendMessage({
            cmd: 'ClearStrongholdPassword',
        });
    }

    async createAccount(payload: CreateAccountPayload): Promise<Account> {
        const response = await this.messageHandler.sendMessage({
            cmd: 'CreateAccount',
            payload,
        });
        return new Account(JSON.parse(response).payload, this.messageHandler);
    }

    async deleteAccountsAndDatabase(): Promise<void> {
        await this.messageHandler.sendMessage({
            cmd: 'DeleteAccountsAndDatabase',
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

    async getLedgerStatus(): Promise<LedgerStatus> {
        const response = await this.messageHandler.sendMessage({
            cmd: 'GetLedgerStatus',
        });
        return JSON.parse(response).payload;
    }

    /**
     * Transform hex encoded address to bech32 encoded address. If no bech32Hrp
     * is provided, the AccountManager will attempt to retrieve it from the
     * NodeInfo. If this does not succeed, it will default to the Shimmer testnet bech32Hrp.
     */
    async hexToBech32(hex: string, bech32Hrp?: string): Promise<string> {
        const response = await this.messageHandler.sendMessage({
            cmd: 'HexToBech32',
            payload: { hex, bech32Hrp },
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

    clearListeners(eventTypes: EventType[]): void {
        return this.messageHandler.clearListeners(eventTypes);
    }

    // TODO: test this
    async recoverAccounts(
        accountGapLimit: number,
        addressGapLimit: number,
        syncOptions: AccountSyncOptions,
    ): Promise<Account[]> {
        const response = await this.messageHandler.sendMessage({
            cmd: 'RecoverAccounts',
            payload: {
                accountGapLimit,
                addressGapLimit,
                syncOptions,
            },
        });
        return JSON.parse(response).payload;
    }

    async removeLatestAccount(): Promise<void> {
        await this.messageHandler.sendMessage({
            cmd: 'RemoveLatestAccount',
        });
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
