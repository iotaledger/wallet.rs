// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import type { MessageHandler } from './MessageHandler';
import type {
    AccountBalance,
    Address,
    AccountSyncOptions,
    AccountMeta,
    OutputsToClaim,
    OutputData,
    Transaction,
    NativeTokenOptions,
    TransactionOptions,
    NftOptions,
    AddressWithAmount,
    AddressWithMicroAmount,
    AddressNativeTokens,
    AddressNftId,
    AddressGenerationOptions,
    AddressWithUnspentOutputs,
    Transaction,
    PreparedTransactionData,
    OutputOptions,
} from '../types';
import type { SignedTransactionEssence } from '../types/signedTransactionEssence';
import type {
    BuildAliasOutputData,
    BuildBasicOutputData,
    BuildFoundryOutputData,
    BuildNftOutputData,
} from '../types/buildOutputData';
import type {
    IAliasOutput,
    IBasicOutput,
    IFoundryOutput,
    INftOutput,
    OutputTypes,
} from '@iota/types';

export class Account {
    meta: AccountMeta;
    private messageHandler: MessageHandler;

    constructor(accountMeta: AccountMeta, messageHandler: MessageHandler) {
        this.meta = accountMeta;
        this.messageHandler = messageHandler;
    }

    async buildAliasOutput(data: BuildAliasOutputData): Promise<IAliasOutput> {
        const resp = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'BuildAliasOutput',
                data,
            },
        );
        return JSON.parse(resp).payload;
    }

    async buildBasicOutput(data: BuildBasicOutputData): Promise<IBasicOutput> {
        const resp = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'BuildBasicOutput',
                data,
            },
        );
        return JSON.parse(resp).payload;
    }

    async buildFoundryOutput(
        data: BuildFoundryOutputData,
    ): Promise<IFoundryOutput> {
        const resp = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'BuildFoundryOutput',
                data,
            },
        );
        return JSON.parse(resp).payload;
    }

    async buildNftOutput(data: BuildNftOutputData): Promise<INftOutput> {
        const resp = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'BuildNftOutput',
                data,
            },
        );
        return JSON.parse(resp).payload;
    }

    async claimOutputs(outputIds: string[]): Promise<Transaction[]> {
        const resp = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'ClaimOutputs',
                data: {
                    outputIdsToClaim: outputIds,
                },
            },
        );
        return JSON.parse(resp).payload;
    }

    async consolidateOutputs(
        force: boolean,
        outputConsolidationThreshold?: number,
    ): Promise<Transaction[]> {
        const resp = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'ConsolidateOutputs',
                data: {
                    force,
                    outputConsolidationThreshold,
                },
            },
        );
        return JSON.parse(resp).payload;
    }

    async generateAddress(
        options?: AddressGenerationOptions,
    ): Promise<Address> {
        const addresses = await this.generateAddresses(1, options);
        return addresses[0];
    }

    async generateAddresses(
        amount: number,
        options?: AddressGenerationOptions,
    ): Promise<Address[]> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'GenerateAddresses',
                data: {
                    amount,
                    options,
                },
            },
        );
        return JSON.parse(response).payload;
    }

    getAlias(): string {
        return this.meta.alias;
    }

    async getBalance(): Promise<AccountBalance> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'GetBalance',
            },
        );

        return JSON.parse(response).payload;
    }

    async getOutput(outputId: string): Promise<OutputData> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'GetOutput',
                data: {
                    outputId,
                },
            },
        );
        return JSON.parse(response).payload;
    }

    async getOutputsWithAdditionalUnlockConditions(
        outputs: OutputsToClaim,
    ): Promise<string[]> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'GetOutputsWithAdditionalUnlockConditions',
                data: {
                    outputsToClaim: outputs,
                },
            },
        );
        return JSON.parse(response).payload;
    }

    async getTransaction(transactionId: string): Promise<Transaction> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'GetTransaction',
                data: {
                    transactionId,
                },
            },
        );
        return JSON.parse(response).payload;
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

    async listAddressesWithUnspentOutputs(): Promise<
        AddressWithUnspentOutputs[]
    > {
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

    async listUnspentOutputs(): Promise<OutputData[]> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'ListUnspentOutputs',
            },
        );

        return JSON.parse(response).payload;
    }

    async minimumRequiredStorageDeposit(
        outputs: OutputTypes[],
    ): Promise<string> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'MinimumRequiredStorageDeposit',
                data: {
                    outputs,
                },
            },
        );
        return JSON.parse(response).payload;
    }

    async mintNativeToken(
        nativeTokenOptions: NativeTokenOptions,
        transactionOptions?: TransactionOptions,
    ): Promise<MintTokenTransaction> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'MintNativeToken',
                data: {
                    nativeTokenOptions: nativeTokenOptions,
                    options: transactionOptions,
                },
            },
        );

        return JSON.parse(response).payload;
    }

    async mintNfts(
        nftsOptions: NftOptions[],
        transactionOptions?: TransactionOptions,
    ): Promise<Transaction> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'MintNfts',
                data: {
                    nftsOptions,
                    options: transactionOptions,
                },
            },
        );

        return JSON.parse(response).payload;
    }

    async prepareOutput(
        options: OutputOptions,
        transactionOptions?: TransactionOptions,
    ): Promise<OutputTypes> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'PrepareOutput',
                data: {
                    options,
                    transactionOptions
                }
            }
        )
        return JSON.parse(response).payload
    }

    async prepareSendAmount(
        addressWithAmount: AddressWithAmount[],
        options?: TransactionOptions,
    ): Promise<PreparedTransactionData> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'PrepareSendAmount',
                data: {
                    addressWithAmount,
                    options,
                },
            },
        );
        return JSON.parse(response).payload;
    }
    
    async prepareTransaction(
        outputs: OutputTypes[],
        options?: TransactionOptions,
    ): Promise<PreparedTransactionData> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'PrepareTransaction',
                data: {
                    outputs,
                    options,
                },
            },
        );
        return JSON.parse(response).payload;
    }

    async sendAmount(
        addressesWithAmount: AddressWithAmount[],
        transactionOptions?: TransactionOptions,
    ): Promise<Transaction> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'SendAmount',
                data: {
                    addressWithAmount: addressesWithAmount,
                    options: transactionOptions,
                },
            },
        );

        return JSON.parse(response).payload;
    }

    async sendMicroTransaction(
        addressesWithMicroAmount: AddressWithMicroAmount[],
        transactionOptions?: TransactionOptions,
    ): Promise<Transaction> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'SendMicroTransaction',
                data: {
                    addressWithMicroAmount: addressesWithMicroAmount,
                    options: transactionOptions,
                },
            },
        );

        return JSON.parse(response).payload;
    }

    async sendNativeTokens(
        addressesNativeTokens: AddressNativeTokens[],
        transactionOptions?: TransactionOptions,
    ): Promise<Transaction> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'SendNativeTokens',
                data: {
                    addressNativeTokens: addressesNativeTokens,
                    options: transactionOptions,
                },
            },
        );

        return JSON.parse(response).payload;
    }

    async sendNft(
        addressesAndNftIds: AddressNftId[],
        transactionOptions?: TransactionOptions,
    ): Promise<Transaction> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'SendNft',
                data: {
                    addressNftIds: addressesAndNftIds,
                    options: transactionOptions,
                },
            },
        );

        return JSON.parse(response).payload;
    }

    async sendOutputs(
        outputs: OutputTypes[],
        transactionOptions?: TransactionOptions,
    ): Promise<Transaction> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'SendOutputs',
                data: {
                    outputs,
                    options: transactionOptions,
                },
            },
        );

        return JSON.parse(response).payload;
    }

    async setAlias(alias: string): Promise<void> {
        await this.messageHandler.callAccountMethod(this.meta.index, {
            name: 'SetAlias',
            data: {
                alias,
            },
        });
    }

    async signTransactionEssence(
        preparedTransactionData: PreparedTransactionData,
    ): Promise<SignedTransactionEssence> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'SignTransactionEssence',
                data: {
                    preparedTransactionData,
                },
            },
        );
        return JSON.parse(response).payload;
    }

    async submitAndStoreTransaction(
        signedTransactionData: SignedTransactionEssence,
    ): Promise<Transaction> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'SubmitAndStoreTransaction',
                data: {
                    signedTransactionData,
                },
            },
        );
        return JSON.parse(response).payload;
    }

    async sync(options?: AccountSyncOptions): Promise<AccountBalance> {
        const resp = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'SyncAccount',
                data: {
                    options: options ?? {}
                },
            },
        );
        return JSON.parse(resp).payload;
    }

    async tryClaimOutputs(
        outputsToClaim: OutputsToClaim,
    ): Promise<Transaction[]> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'TryClaimOutputs',
                data: {
                    outputsToClaim,
                },
            },
        );

        return JSON.parse(response).payload;
    }
}
