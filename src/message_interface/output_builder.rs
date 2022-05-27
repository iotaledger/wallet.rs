// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_client::{
    bee_block::{
        output::{
            dto::{NativeTokenDto, OutputDto},
            feature::dto::FeatureDto,
            unlock_condition::dto::UnlockConditionDto,
            AliasId, AliasOutputBuilder, BasicOutputBuilder, Feature, FoundryOutputBuilder, NativeToken, NftId,
            NftOutputBuilder, TokenScheme, UnlockCondition,
        },
        DtoError,
    },
    Client,
};

use crate::Result;

#[allow(clippy::too_many_arguments)]
pub(crate) async fn build_basic_output(
    client: Client,
    // If not provided, minimum storage deposit will be used
    amount: Option<String>,
    native_tokens: Option<Vec<NativeTokenDto>>,
    unlock_conditions: Vec<UnlockConditionDto>,
    features: Option<Vec<FeatureDto>>,
) -> Result<OutputDto> {
    let mut builder: BasicOutputBuilder;
    if let Some(amount) = amount {
        builder =
            BasicOutputBuilder::new_with_amount(amount.parse::<u64>().map_err(|_| DtoError::InvalidField("amount"))?)?;
    } else {
        // Config Builder
        let byte_cost_config = client.get_byte_cost_config().await?;
        builder = BasicOutputBuilder::new_with_minimum_storage_deposit(byte_cost_config)?;
    }

    if let Some(native_tokens) = native_tokens {
        let tokens = native_tokens
            .iter()
            .map(|native_token| Ok(NativeToken::try_from(native_token)?))
            .collect::<Result<Vec<NativeToken>>>()?;
        builder = builder.with_native_tokens(tokens);
    }

    let conditions = unlock_conditions
        .iter()
        .map(|unlock_condition| Ok(UnlockCondition::try_from(unlock_condition)?))
        .collect::<Result<Vec<UnlockCondition>>>()?;
    builder = builder.with_unlock_conditions(conditions);

    if let Some(features) = features {
        let blocks = features
            .iter()
            .map(|feature| Ok(Feature::try_from(feature)?))
            .collect::<Result<Vec<Feature>>>()?;
        builder = builder.with_features(blocks);
    }

    let basic_output = builder.finish_output()?;

    // Convert to Dto
    Ok(OutputDto::from(&basic_output))
}

// #[allow(clippy::too_many_arguments)]
// fn build_alias_output(
//     alias_id: String,
//     amount: Option<u64>,
//     byte_cost: Option<u64>,
//     key_factor: Option<u64>,
//     data_factor: Option<u64>,
//     native_tokens: Option<Vec<String>>,
//     state_index: Option<u32>,
//     state_metadata: Option<Vec<u8>>,
//     foundry_counter: Option<u32>,
//     unlock_conditions: Option<Vec<String>>,
//     features: Option<Vec<String>>,
//     immutable_features: Option<Vec<String>>,
// ) -> Result<OutputDto> {
//     let id = serde_json::from_str::<AliasId>(&alias_id).unwrap_or_else(|_| panic!("Invalid AliasId: {:?}",
// alias_id));     let mut builder: AliasOutputBuilder;
//     if let Some(amount) = amount {
//         builder = AliasOutputBuilder::new_with_amount(amount, id)?;
//     } else {
//         // Config Builder
//         let mut config_builder = ByteCostConfigBuilder::new();
//         if let Some(byte_cost) = byte_cost {
//             config_builder = config_builder.byte_cost(byte_cost)
//         }
//         if let Some(key_factor) = key_factor {
//             config_builder = config_builder.key_factor(key_factor)
//         }
//         if let Some(data_factor) = data_factor {
//             config_builder = config_builder.data_factor(data_factor)
//         }
//         let config = config_builder.finish();
//         builder = AliasOutputBuilder::new_with_minimum_storage_deposit(config, id)?;
//     }
//     if let Some(native_tokens) = native_tokens {
//         let tokens: Vec<NativeToken> = native_tokens
//             .iter()
//             .map(|native_token| {
//                 serde_json::from_str::<NativeToken>(native_token)
//                     .unwrap_or_else(|_| panic!("Invalid NativeToken: {:?}", native_token))
//             })
//             .collect();
//         builder = builder.with_native_tokens(tokens);
//     }
//     if let Some(state_index) = state_index {
//         builder = builder.with_state_index(state_index);
//     }
//     if let Some(state_metadata) = state_metadata {
//         builder = builder.with_state_metadata(state_metadata);
//     }
//     if let Some(foundry_counter) = foundry_counter {
//         builder = builder.with_foundry_counter(foundry_counter);
//     }
//     if let Some(unlock_conditions) = unlock_conditions {
//         let conditions: Vec<UnlockCondition> = unlock_conditions
//             .iter()
//             .map(|unlock_condition| {
//                 serde_json::from_str::<UnlockCondition>(unlock_condition)
//                     .unwrap_or_else(|_| panic!("Invalid UnlockCondition: {:?}", unlock_condition))
//             })
//             .collect();
//         builder = builder.with_unlock_conditions(conditions);
//     }
//     if let Some(features) = features {
//         let blocks: Vec<Feature> = features
//             .iter()
//             .map(|feature| {
//                 serde_json::from_str::<Feature>(feature).unwrap_or_else(|_| panic!("Invalid Feature: {:?}", feature))
//             })
//             .collect();
//         builder = builder.with_features(blocks);
//     }
//     if let Some(immutable_features) = immutable_features {
//         let blocks: Vec<Feature> = immutable_features
//             .iter()
//             .map(|immutable_feature| {
//                 serde_json::from_str::<Feature>(immutable_feature)
//                     .unwrap_or_else(|_| panic!("Invalid immutable Feature: {:?}", immutable_feature))
//             })
//             .collect();
//         builder = builder.with_immutable_features(blocks);
//     }
//     let alias_output = builder.finish_output()?;
//     // Convert to Dto
//     Ok(OutputDto::from(&alias_output))
// }

// #[allow(clippy::too_many_arguments)]
// fn build_foundry_output(
//     serial_number: u32,
//     token_scheme: String,
//     amount: Option<u64>,
//     byte_cost: Option<u64>,
//     key_factor: Option<u64>,
//     data_factor: Option<u64>,
//     native_tokens: Option<Vec<String>>,
//     unlock_conditions: Option<Vec<String>>,
//     features: Option<Vec<String>>,
//     immutable_features: Option<Vec<String>>,
// ) -> Result<OutputDto> {
//     let scheme = serde_json::from_str::<TokenScheme>(&token_scheme)
//         .unwrap_or_else(|_| panic!("Invalid TokenScheme: {:?}", token_scheme));
//     let mut builder: FoundryOutputBuilder;
//     if let Some(amount) = amount {
//         builder = FoundryOutputBuilder::new_with_amount(amount, serial_number, scheme)?;
//     } else {
//         // Config Builder
//         let mut config_builder = ByteCostConfigBuilder::new();
//         if let Some(byte_cost) = byte_cost {
//             config_builder = config_builder.byte_cost(byte_cost)
//         }
//         if let Some(key_factor) = key_factor {
//             config_builder = config_builder.key_factor(key_factor)
//         }
//         if let Some(data_factor) = data_factor {
//             config_builder = config_builder.data_factor(data_factor)
//         }
//         let config = config_builder.finish();
//         builder = FoundryOutputBuilder::new_with_minimum_storage_deposit(config, serial_number, scheme)?;
//     }
//     if let Some(native_tokens) = native_tokens {
//         let tokens: Vec<NativeToken> = native_tokens
//             .iter()
//             .map(|native_token| {
//                 serde_json::from_str::<NativeToken>(native_token)
//                     .unwrap_or_else(|_| panic!("Invalid NativeToken: {:?}", native_token))
//             })
//             .collect();
//         builder = builder.with_native_tokens(tokens);
//     }
//     if let Some(unlock_conditions) = unlock_conditions {
//         let conditions: Vec<UnlockCondition> = unlock_conditions
//             .iter()
//             .map(|unlock_condition| {
//                 serde_json::from_str::<UnlockCondition>(unlock_condition)
//                     .unwrap_or_else(|_| panic!("Invalid UnlockCondition: {:?}", unlock_condition))
//             })
//             .collect();
//         builder = builder.with_unlock_conditions(conditions);
//     }
//     if let Some(features) = features {
//         let blocks: Vec<Feature> = features
//             .iter()
//             .map(|feature| {
//                 serde_json::from_str::<Feature>(feature).unwrap_or_else(|_| panic!("Invalid Feature: {:?}", feature))
//             })
//             .collect();
//         builder = builder.with_features(blocks);
//     }
//     if let Some(immutable_features) = immutable_features {
//         let blocks: Vec<Feature> = immutable_features
//             .iter()
//             .map(|immutable_feature| {
//                 serde_json::from_str::<Feature>(immutable_feature)
//                     .unwrap_or_else(|_| panic!("Invalid immutable Feature: {:?}", immutable_feature))
//             })
//             .collect();
//         builder = builder.with_immutable_features(blocks);
//     }
//     let foundry_output = builder.finish_output()?;
//     // Convert to Dto
//     Ok(OutputDto::from(&foundry_output))
// }

// #[allow(clippy::too_many_arguments)]
// fn build_nft_output(
//     nft_id: String,
//     amount: Option<u64>,
//     byte_cost: Option<u64>,
//     key_factor: Option<u64>,
//     data_factor: Option<u64>,
//     native_tokens: Option<Vec<String>>,
//     unlock_conditions: Option<Vec<String>>,
//     features: Option<Vec<String>>,
//     immutable_features: Option<Vec<String>>,
// ) -> Result<OutputDto> {
//     let id = serde_json::from_str::<NftId>(&nft_id).unwrap_or_else(|_| panic!("Invalid NftId: {:?}", nft_id));
//     let mut builder: NftOutputBuilder;
//     if let Some(amount) = amount {
//         builder = NftOutputBuilder::new_with_amount(amount, id)?;
//     } else {
//         // Config Builder
//         let mut config_builder = ByteCostConfigBuilder::new();
//         if let Some(byte_cost) = byte_cost {
//             config_builder = config_builder.byte_cost(byte_cost)
//         }
//         if let Some(key_factor) = key_factor {
//             config_builder = config_builder.key_factor(key_factor)
//         }
//         if let Some(data_factor) = data_factor {
//             config_builder = config_builder.data_factor(data_factor)
//         }
//         let config = config_builder.finish();
//         builder = NftOutputBuilder::new_with_minimum_storage_deposit(config, id)?;
//     }
//     if let Some(native_tokens) = native_tokens {
//         let tokens: Vec<NativeToken> = native_tokens
//             .iter()
//             .map(|native_token| {
//                 serde_json::from_str::<NativeToken>(native_token)
//                     .unwrap_or_else(|_| panic!("Invalid NativeToken: {:?}", native_token))
//             })
//             .collect();
//         builder = builder.with_native_tokens(tokens);
//     }
//     if let Some(unlock_conditions) = unlock_conditions {
//         let conditions: Vec<UnlockCondition> = unlock_conditions
//             .iter()
//             .map(|unlock_condition| {
//                 serde_json::from_str::<UnlockCondition>(unlock_condition)
//                     .unwrap_or_else(|_| panic!("Invalid UnlockCondition: {:?}", unlock_condition))
//             })
//             .collect();
//         builder = builder.with_unlock_conditions(conditions);
//     }
//     if let Some(features) = features {
//         let blocks: Vec<Feature> = features
//             .iter()
//             .map(|feature| {
//                 serde_json::from_str::<Feature>(feature).unwrap_or_else(|_| panic!("Invalid Feature: {:?}", feature))
//             })
//             .collect();
//         builder = builder.with_features(blocks);
//     }
//     if let Some(immutable_features) = immutable_features {
//         let blocks: Vec<Feature> = immutable_features
//             .iter()
//             .map(|immutable_feature| {
//                 serde_json::from_str::<Feature>(immutable_feature)
//                     .unwrap_or_else(|_| panic!("Invalid immutable Feature: {:?}", immutable_feature))
//             })
//             .collect();
//         builder = builder.with_immutable_features(blocks);
//     }
//     let nft_output = builder.finish_output()?;
//     // Convert to Dto
//     Ok(OutputDto::from(&nft_output))
// }
