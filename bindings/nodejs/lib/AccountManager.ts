// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { MessageHandler } from './MessageHandler';
import { Account } from './Account';

import type {
    AccountId,
    AccountManagerOptions,
    AccountSyncOptions,
    Auth,
    ClientOptions,
    CreateAccountPayload,
    EventType,
    GenerateAddressOptions,
    LedgerNanoStatus,
    NodeInfoWrapper,
    WalletEvent,
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
            cmd: 'backup',
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
            cmd: 'bech32ToHex',
            payload: { bech32Address },
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
            cmd: 'changeStrongholdPassword',
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
            cmd: 'clearStrongholdPassword',
        });
    }

    /**
     * Create a new account.
     */
    async createAccount(payload: CreateAccountPayload): Promise<Account> {
        const response = await this.messageHandler.sendMessage({
            cmd: 'createAccount',
            payload,
        });
        return new Account(JSON.parse(response).payload, this.messageHandler);
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
            cmd: 'emitTestEvent',
            payload: { event },
        });
    }

    /**
     * Generate a random BIP39 mnemonic.
     */
    async generateMnemonic(): Promise<string> {
        const response = await this.messageHandler.sendMessage({
            cmd: 'generateMnemonic',
        });
        return JSON.parse(response).payload;
    }

    /**
     * Get an account by its alias or index.
     */
    async getAccount(accountId: AccountId): Promise<Account> {
        const response = await this.messageHandler.sendMessage({
            cmd: 'getAccount',
            payload: { accountId },
        });

        const account = new Account(
            JSON.parse(response).payload,
            this.messageHandler,
        );

        return account;
    }

    /**
     * Get all account indexes.
     */
    async getAccountIndexes(): Promise<number[]> {
        const response = await this.messageHandler.sendMessage({
            cmd: 'getAccountIndexes',
        });

        return JSON.parse(response).payload;
    }

    /**
     * Get all accounts.
     */
    async getAccounts(): Promise<Account[]> {
        const response = await this.messageHandler.sendMessage({
            cmd: 'getAccounts',
        });

        const { payload } = JSON.parse(response);

        const accounts: Account[] = [];

        for (const account of payload) {
            accounts.push(new Account(account, this.messageHandler));
        }
        return accounts;
    }

    /**
     * Generate an address without storing it.
     */
    async generateAddress(
        accountIndex: number,
        internal: boolean,
        addressIndex: number,
        options?: GenerateAddressOptions,
        bech32Hrp?: string,
    ): Promise<string> {
        const response = await this.messageHandler.sendMessage({
            cmd: 'generateAddress',
            payload: {
                accountIndex,
                internal,
                addressIndex,
                options,
                bech32Hrp,
            },
        });
        return JSON.parse(response).payload;
    }

    /**
     * Get the node info.
     */
    async getNodeInfo(url?: string, auth?: Auth): Promise<NodeInfoWrapper> {
        const response = await this.messageHandler.sendMessage({
            cmd: 'getNodeInfo',
            payload: { url, auth },
        });
        return JSON.parse(response).payload;
    }

    /**
     * Get the status for a Ledger Nano.
     */
    async getLedgerNanoStatus(): Promise<LedgerNanoStatus> {
        const response = await this.messageHandler.sendMessage({
            cmd: 'getLedgerNanoStatus',
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
            cmd: 'hexToBech32',
            payload: { hex, bech32Hrp },
        });
        return JSON.parse(response).payload;
    }

    /**
     * Check if the Stronghold password has been set.
     */
    async isStrongholdPasswordAvailable(): Promise<boolean> {
        const response = await this.messageHandler.sendMessage({
            cmd: 'isStrongholdPasswordAvailable',
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
    async clearListeners(eventTypes: EventType[]): Promise<void> {
        const response = await this.messageHandler.sendMessage({
            cmd: 'clearListeners',
            payload: { eventTypes },
        });
        return JSON.parse(response).payload;
    }

    /**
     * Find accounts with unspent outputs.
     */
    async recoverAccounts(
        accountStartIndex: number,
        accountGapLimit: number,
        addressGapLimit: number,
        syncOptions: AccountSyncOptions,
    ): Promise<Account[]> {
        const response = await this.messageHandler.sendMessage({
            cmd: 'recoverAccounts',
            payload: {
                accountStartIndex,
                accountGapLimit,
                addressGapLimit,
                syncOptions,
            },
        });

        const accounts: Account[] = [];

        for (const account of JSON.parse(response).payload) {
            accounts.push(new Account(account, this.messageHandler));
        }
        return accounts;
    }

    /**
     * Delete the latest account.
     */
    async removeLatestAccount(): Promise<void> {
        await this.messageHandler.sendMessage({
            cmd: 'removeLatestAccount',
        });
    }

    /**
     * Restore a backup from a Stronghold file
     * Replaces client_options, coin_type, secret_manager and accounts. Returns an error if accounts were already created
     * If Stronghold is used as secret_manager, the existing Stronghold file will be overwritten. If a mnemonic was
     * stored, it will be gone.
     */
    async restoreBackup(source: string, password: string): Promise<void> {
        await this.messageHandler.sendMessage({
            cmd: 'restoreBackup',
            payload: {
                source,
                password,
            },
        });
    }

    /**
     * Set ClientOptions.
     */
    async setClientOptions(clientOptions: ClientOptions): Promise<void> {
        await this.messageHandler.sendMessage({
            cmd: 'setClientOptions',
            payload: { clientOptions },
        });
    }

    /**
     * Set the Stronghold password.
     */
    async setStrongholdPassword(password: string): Promise<void> {
        await this.messageHandler.sendMessage({
            cmd: 'setStrongholdPassword',
            payload: { password },
        });
    }

    /**
     * Set the interval after which the Stronghold password gets cleared from memory.
     */
    async setStrongholdPasswordClearInterval(
        intervalInMilliseconds?: number,
    ): Promise<void> {
        await this.messageHandler.sendMessage({
            cmd: 'setStrongholdPasswordClearInterval',
            payload: { intervalInMilliseconds },
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
            cmd: 'startBackgroundSync',
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
            cmd: 'stopBackgroundSync',
        });
    }

    /**
     * Store a mnemonic in the Stronghold snapshot.
     */
    async storeMnemonic(mnemonic: string): Promise<void> {
        await this.messageHandler.sendMessage({
            cmd: 'storeMnemonic',
            payload: { mnemonic },
        });
    }

    /**
     * Verify if a mnemonic is a valid BIP39 mnemonic.
     */
    async verifyMnemonic(mnemonic: string): Promise<void> {
        await this.messageHandler.sendMessage({
            cmd: 'verifyMnemonic',
            payload: { mnemonic },
        });
    }
}
