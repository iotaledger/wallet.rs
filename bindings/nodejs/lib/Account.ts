// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import type { MessageHandler } from './MessageHandler';
import type {
    AccountBalance,
    Address,
    AccountSyncOptions,
    AccountMeta,
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
    AddressGenerationOptions,
    AddressWithUnspentOutputs,
    TransactionReceipt
} from '../types';

export class Account {
    meta: AccountMeta;
    private messageHandler: MessageHandler;

    constructor(accountMeta: AccountMeta, messageHandler: MessageHandler) {
        this.meta = accountMeta;
        this.messageHandler = messageHandler;
    }

    async collectOutputs(outputIds: string[]): Promise<TransactionReceipt[]> {
        const resp = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'CollectOutputs',
                data: {
                    outputIdsToCollect: outputIds
                },
            }
        )
        return JSON.parse(resp).payload;
    }

    getAlias(): string {
        return this.meta.alias;
    }

    async getBalance(): Promise<AccountBalance> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'GetBalance',
            }
        )

        return JSON.parse(response).payload;
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

    async listAddressesWithUnspentOutputs(): Promise<AddressWithUnspentOutputs[]> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'ListAddressesWithUnspentOutputs',
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
            this.meta.index, {
            name: 'ListPendingTransactions'
        })
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
                    options,
                }
            }
        )

        return JSON.parse(response).payload;
    }

    async mintNativeToken(
        nativeTokenOptions: NativeTokenOptions,
        transferOptions?: TransferOptions,
    ): Promise<TransactionReceipt[]> {
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
        nftsOptions: NftOptions[],
        transferOptions?: TransferOptions,
    ): Promise<TransactionReceipt[]> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'MintNfts',
                data: {
                    nftsOptions,
                    options: transferOptions,
                },
            },
        );

        return JSON.parse(response).payload;
    }

    async sendAmount(
        addressesWithAmount: AddressWithAmount[],
        transferOptions?: TransferOptions,
    ): Promise<TransactionReceipt[]> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'SendAmount',
                data: {
                    addressWithAmount: addressesWithAmount,
                    options: transferOptions
                }
            },
        );

        return JSON.parse(response).payload;
    }

    async sendMicroTransaction(
        addressesWithMicroAmount: AddressWithMicroAmount[],
        transferOptions?: TransferOptions,
    ): Promise<TransactionReceipt[]> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'SendMicroTransaction',
                data: {
                    addressWithMicroAmount: addressesWithMicroAmount,
                    options: transferOptions
                }
            }
        )

        return JSON.parse(response).payload;
    }

    async sendNativeTokens(addressesNativeTokens: AddressNativeTokens[], transferOptions?: TransferOptions): Promise<TransactionReceipt[]> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'SendNativeTokens',
                data: {
                    addressNativeTokens: addressesNativeTokens,
                    options: transferOptions
                }
            }
        )

        return JSON.parse(response).payload;
    }

    async sendNft(
        addressesAndNftIds: AddressNftId[],
        transferOptions?: TransferOptions,
    ): Promise<TransactionReceipt[]> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'SendNft',
                data: {
                    addressNftIds: addressesAndNftIds,
                    options: transferOptions
                }
            }
        )

        return JSON.parse(response).payload;
    }

    async sendTransfer(
        outputs: OutputData[],
        transferOptions?: TransferOptions,
    ): Promise<TransactionReceipt[]> {
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

    async tryCollectOutputs(outputsToCollect: OutputsToCollect): Promise<TransactionReceipt[]> {
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
