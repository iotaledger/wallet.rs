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

    /**
     * Backup the data to a Stronghold snapshot.
     */
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

    /**
     * Change the Stronghold password.
     */
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

    /**
     * Clear the Stronghold password from memory.
     */
    async clearStrongholdPassword(): Promise<void> {
        await this.messageHandler.sendMessage({
            cmd: 'ClearStrongholdPassword',
        });
    }

    /**
     * Create a new account.
     */
    async createAccount(payload: CreateAccountPayload): Promise<Account> {
        const response = await this.messageHandler.sendMessage({
            cmd: 'CreateAccount',
            payload,
        });
        return new Account(JSON.parse(response).payload, this.messageHandler);
    }

    /**
     * Delete all accounts and the database folder.
     */
    async deleteAccountsAndDatabase(): Promise<void> {
        await this.messageHandler.sendMessage({
            cmd: 'DeleteAccountsAndDatabase',
        });
    }

    /**
     * Destroy the AccountManager and drop its database connection.
     */
    destroy(): void {
        this.messageHandler.destroy();
    }

    /**
     * Emit a provided event for testing of the event system.
     */
    async emitTestEvent(event: WalletEvent): Promise<void> {
        await this.messageHandler.sendMessage({
            cmd: 'EmitTestEvent',
            payload: event,
        });
    }

    /**
     * Generate a random BIP39 mnemonic.
     */
    async generateMnemonic(): Promise<string> {
        const response = await this.messageHandler.sendMessage({
            cmd: 'GenerateMnemonic',
        });
        return JSON.parse(response).payload;
    }

    /**
     * Get an account by it's alias or index.
     */
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

    /**
     * Get all accounts.
     */
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

    /**
     * Get the node info.
     */
    async getNodeInfo(url?: string, auth?: Auth): Promise<NodeInfoWrapper> {
        const response = await this.messageHandler.sendMessage({
            cmd: 'GetNodeInfo',
            payload: { url, auth },
        });
        return JSON.parse(response).payload;
    }

    /**
     * Get the status for a Ledger Nano.
     */
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

    /**
     * Check if the Stronghold password has been set.
     */
    async isStrongholdPasswordAvailable(): Promise<boolean> {
        const response = await this.messageHandler.sendMessage({
            cmd: 'IsStrongholdPasswordAvailable',
        });
        return JSON.parse(response).payload;
    }

    /**
     * Listen to wallet events with a callback. An empty array will listen to all possible events.
     */
    listen(
        eventTypes: EventType[],
        callback: (error: Error, result: string) => void,
    ): void {
        return this.messageHandler.listen(eventTypes, callback);
    }

    /**
     * Clear the callbacks for provided events. An empty array will clear all listeners.
     */
    clearListeners(eventTypes: EventType[]): void {
        return this.messageHandler.clearListeners(eventTypes);
    }

    /**
     * Find accounts with unspent outputs.
     */
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

    /**
     * Delete the latest account.
     */
    async removeLatestAccount(): Promise<void> {
        await this.messageHandler.sendMessage({
            cmd: 'RemoveLatestAccount',
        });
    }

    /**
     * Restore a backup from a Stronghold snapshot.
     */
    async restoreBackup(source: string, password: string): Promise<void> {
        await this.messageHandler.sendMessage({
            cmd: 'RestoreBackup',
            payload: {
                source,
                password,
            },
        });
    }

    /**
     * Set ClientOptions.
     */
    async setClientOptions(options: ClientOptions): Promise<void> {
        await this.messageHandler.sendMessage({
            cmd: 'SetClientOptions',
            payload: options,
        });
    }

    /**
     * Set the Stronghold password.
     */
    async setStrongholdPassword(password: string): Promise<void> {
        await this.messageHandler.sendMessage({
            cmd: 'SetStrongholdPassword',
            payload: password,
        });
    }

    /**
     * Set the interval after which the Stronghold password gets cleared from memory.
     */
    async setStrongholdPasswordClearInterval(
        intervalInMilliseconds?: number,
    ): Promise<void> {
        await this.messageHandler.sendMessage({
            cmd: 'SetStrongholdPasswordClearInterval',
            payload: intervalInMilliseconds,
        });
    }

    /**
     * Start the background syncing process for all accounts.
     */
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

    /**
     * Stop the background syncing process for all accounts.
     */
    async stopBackgroundSync(): Promise<void> {
        await this.messageHandler.sendMessage({
            cmd: 'StopBackgroundSync',
        });
    }

    /**
     * Store a mnemonic in the Stronghold snapshot.
     */
    async storeMnemonic(mnemonic: string): Promise<void> {
        await this.messageHandler.sendMessage({
            cmd: 'StoreMnemonic',
            payload: mnemonic,
        });
    }

    /**
     * Verify if a mnemonic is a valid BIP39 mnemonic.
     */
    async verifyMnemonic(mnemonic: string): Promise<void> {
        await this.messageHandler.sendMessage({
            cmd: 'VerifyMnemonic',
            payload: mnemonic,
        });
    }
}
