import type { FeatureTypes, INativeToken, TokenSchemeTypes, UnlockConditionTypes } from '@iota/types';

export interface BuildAliasOutputData extends BuildBasicOutputData {
    aliasId: string;
    stateIndex?: number;
    stateMetadata?: number[];
    foundryCounter?: number;
    immutableFeatures?: FeatureTypes[]
}

export interface BuildBasicOutputData {
    amount?: string;
    nativeTokens?: INativeToken;
    unlockConditions: UnlockConditionTypes[];
    features?: FeatureTypes[];
}

export interface BuildFoundryOutputData extends BuildBasicOutputData {
    serialNumber: number;
    tokenScheme: TokenSchemeTypes;
    immutableFeatures?: FeatureTypes[]
}

export interface BuildNftOutputData extends BuildBasicOutputData {
    nftId: string;
    immutableFeatures?: FeatureTypes[]
}

