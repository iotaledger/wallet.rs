// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import type { MessageHandler } from './MessageHandler';
import type {
    AccountBalance,
    Address,
    AccountSyncOptions,
    AccountMeta,
    ClientOptions,
    OutputsToCollect,
    OutputData,
    Transaction,
    NativeTokenOptions,
    TransferOptions,
    NftOptions,
    AddressWithAmount,
    AddressWithMicroAmount,
    AddressNativeTokens,
    AddressNftId,
    AddressGenerationOptions
} from '../types';

export class Account {
    meta: AccountMeta;
    private messageHandler: MessageHandler;

    constructor(accountMeta: AccountMeta, messageHandler: MessageHandler) {
        this.meta = accountMeta;
        this.messageHandler = messageHandler;
    }

    alias(): string {
        return this.meta.alias;
    }

    async collectOutputs(outputIds: string[]): Promise<void> {
        await this.messageHandler.callAccountMethod(this.meta.index, {
            name: 'CollectOutputs',
            data: {
                outputIdsToCollect: outputIds,
            },
        });
    }

    async getOutput(outputId: string): Promise<OutputData> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'GetOutput',
                data: {
                    outputId
                }
            }
        )
        return JSON.parse(response).payload;
    }
    
    async getOutputsWithAdditionalUnlockConditions(outputs: OutputsToCollect): Promise<string> {
        return await this.messageHandler.callAccountMethod(this.meta.index, {
            name: 'GetOutputsWithAdditionalUnlockConditions',
            data: {
                outputsToCollect: outputs,
            },
        });
    }

    async listAddresses(): Promise<Address[]> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'ListAddresses',
            },
        );

        return JSON.parse(response).payload;
    }

    async listAddressesWithBalance(): Promise<Address[]> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'ListAddressesWithBalance',
            },
        );

        return JSON.parse(response).payload;
    }

    async listOutputs(): Promise<OutputData[]> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'ListOutputs',
            },
        );

        return JSON.parse(response).payload;
    }

    async listUnspentOutputs(): Promise<OutputData[]> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'ListUnspentOutputs',
            },
        );

        return JSON.parse(response).payload;
    }

    async listPendingTransactions(): Promise<Transaction[]> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'ListPendingTransactions',
            },
        );

        return JSON.parse(response).payload;
    }

    async listTransactions(): Promise<Transaction[]> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'ListTransactions',
            },
        );

        return JSON.parse(response).payload;
    }

    async sync(options?: AccountSyncOptions): Promise<void> {
        await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'SyncAccount',
                data: options ?? {},
            }
        )
    }

    async generateAddress(options?: AddressGenerationOptions): Promise<Address> {
        const addresses = await this.generateAddresses(1, options);
        return addresses[0];
    }

    async generateAddresses(amount: number, options?: AddressGenerationOptions): Promise<Address[]> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'GenerateAddresses',
                data: {
                    amount,
                    options
                }
            }
        )

        return JSON.parse(response).payload;
    }

    async latestAddress(): Promise<Address> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'GetLatestAddress',
            },
        );

        return JSON.parse(response).payload;
    }

    async balance(): Promise<AccountBalance> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'GetBalance',
            },
        );

        return JSON.parse(response).payload;
    }

    async mintNativeToken(
        nativeTokenOptions: NativeTokenOptions,
        transferOptions: TransferOptions,
    ): Promise<Transaction[]> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'MintNativeToken',
                data: {
                    nativeTokenOptions: nativeTokenOptions,
                    options: transferOptions,
                },
            },
        );

        return JSON.parse(response).payload;
    }

    async mintNfts(
        nftOptions: NftOptions,
        transferOptions: TransferOptions,
    ): Promise<Transaction[]> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'MintNfts',
                data: {
                    nftsOptions: nftOptions,
                    options: transferOptions,
                },
            },
        );

        return JSON.parse(response).payload;
    }

    async sendAmount(
        addressesWithAmount: AddressWithAmount[],
        transferOptions: TransferOptions,
    ): Promise<Transaction[]> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'SendAmount',
                data: {
                    addressesWithAmount,
                    options: transferOptions,
                },
            },
        );

        return JSON.parse(response).payload;
    }

    async sendMicroTransaction(
        addressesWithMicroAmount: AddressWithMicroAmount[],
        transferOptions: TransferOptions,
    ): Promise<Transaction[]> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'SendMicroTransaction',
                data: {
                    addressesWithMicroAmount: addressesWithMicroAmount,
                    options: transferOptions,
                },
            },
        );

        return JSON.parse(response).payload;
    }

    async sendNativeTokens(
        addressNativeTokens: AddressNativeTokens[],
        transferOptions: TransferOptions,
    ): Promise<Transaction[]> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'SendNativeTokens',
                data: {
                    addressesNativeTokens: addressNativeTokens,
                    options: transferOptions,
                },
            },
        );

        return JSON.parse(response).payload;
    }

    async sendNft(
        addressesAndNftIds: AddressNftId[],
        transferOptions: TransferOptions,
    ): Promise<Transaction[]> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'SendNft',
                data: {
                    addressesNftIds: addressesAndNftIds,
                    options: transferOptions,
                },
            },
        );

        return JSON.parse(response).payload;
    }

    async sendTransfer(
        outputs: OutputData[],
        transferOptions: TransferOptions,
    ): Promise<Transaction[]> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'SendTransfer',
                data: {
                    outputs,
                    options: transferOptions,
                },
            },
        );

        return JSON.parse(response).payload;
    }

    async tryCollectOutputs(
        outputsToCollect: OutputsToCollect,
    ): Promise<Transaction[]> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'TryCollectOutputs',
                data: {
                    outputsToCollect,
                },
            },
        );

        return JSON.parse(response).payload;
    }
}
