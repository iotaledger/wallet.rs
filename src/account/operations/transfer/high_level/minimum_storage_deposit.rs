// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_client::bee_message::{
    address::{Address, AliasAddress},
    output::{
        unlock_condition::{
            AddressUnlockCondition, ExpirationUnlockCondition, GovernorAddressUnlockCondition,
            ImmutableAliasAddressUnlockCondition, StateControllerAddressUnlockCondition,
            StorageDepositReturnUnlockCondition, UnlockCondition,
        },
        AliasId, AliasOutputBuilder, BasicOutputBuilder, ByteCost, ByteCostConfig, FeatureBlock, FoundryOutputBuilder,
        NativeToken, NftId, NftOutputBuilder, Output, OutputAmount, SimpleTokenScheme, TokenId, TokenScheme, TokenTag,
    },
    payload::milestone::MilestoneIndex,
};
use primitive_types::U256;

use crate::Result;

// todo: move to bee-message/iota.rs

/// Computes the minimum amount that an alias output needs to have.
pub(crate) fn minimum_storage_deposit_alias(config: &ByteCostConfig, address: &Address) -> Result<u64> {
    // Safety: This can never fail because the amount will always be within the valid range. Also, the actual value is
    // not important, we are only interested in the storage requirements of the type.
    let alias_output = AliasOutputBuilder::new_with_amount(OutputAmount::MIN, AliasId::null())?
        .with_state_index(0)
        .with_foundry_counter(0)
        .add_unlock_condition(UnlockCondition::StateControllerAddress(
            StateControllerAddressUnlockCondition::new(*address),
        ))
        .add_unlock_condition(UnlockCondition::GovernorAddress(GovernorAddressUnlockCondition::new(
            *address,
        )))
        .finish()?;
    Ok(Output::Alias(alias_output).byte_cost(config))
}

/// Computes the minimum amount that an foundry output needs to have.
pub(crate) fn minimum_storage_deposit_foundry(config: &ByteCostConfig) -> Result<u64> {
    // Safety: This can never fail because the amount will always be within the valid range. Also, the actual value is
    // not important, we are only interested in the storage requirements of the type.
    let foundry_output = FoundryOutputBuilder::new_with_amount(
        OutputAmount::MIN,
        1,
        TokenTag::new([0u8; 12]),
        TokenScheme::Simple(SimpleTokenScheme::new(U256::from(0), U256::from(0), U256::from(1))?),
    )?
    .add_unlock_condition(UnlockCondition::ImmutableAliasAddress(
        ImmutableAliasAddressUnlockCondition::new(AliasAddress::new(AliasId::new([0u8; AliasId::LENGTH]))),
    ))
    .finish()?;
    Ok(Output::Foundry(foundry_output).byte_cost(config))
}

/// Computes the minimum amount that an output needs to have, when sent with [AddressUnlockCondition],
/// [StorageDepositReturnUnlockCondition] and [ExpirationUnlockCondition].
pub(crate) fn minimum_storage_deposit_basic_native_tokens(
    config: &ByteCostConfig,
    address: &Address,
    return_address: &Address,
    native_tokens: Option<Vec<(TokenId, U256)>>,
) -> Result<u64> {
    let address_condition = UnlockCondition::Address(AddressUnlockCondition::new(*address));
    // Safety: This can never fail because the amount will always be within the valid range. Also, the actual value is
    // not important, we are only interested in the storage requirements of the type.
    let mut basic_output_builder = BasicOutputBuilder::new_with_amount(OutputAmount::MIN)?
        .add_unlock_condition(address_condition)
        .add_unlock_condition(UnlockCondition::StorageDepositReturn(
            StorageDepositReturnUnlockCondition::new(*return_address, OutputAmount::MIN)?,
        ))
        .add_unlock_condition(UnlockCondition::Expiration(ExpirationUnlockCondition::new(
            *return_address,
            // Both 0 would be invalid, so we just use 1
            MilestoneIndex::new(1),
            0,
        )?));
    if let Some(native_tokens) = native_tokens {
        basic_output_builder = basic_output_builder.with_native_tokens(
            native_tokens
                .iter()
                .map(|(id, amount)| {
                    NativeToken::new(*id, *amount).map_err(|e| crate::Error::ClientError(Box::new(e.into())))
                })
                .collect::<Result<Vec<NativeToken>>>()?,
        );
    }
    Ok(Output::Basic(basic_output_builder.finish()?).byte_cost(config))
}

/// Computes the minimum amount that an nft output needs to have.
pub(crate) fn minimum_storage_deposit_nft(
    config: &ByteCostConfig,
    address: &Address,
    immutable_metadata: Option<FeatureBlock>,
    metadata: Option<FeatureBlock>,
) -> crate::Result<u64> {
    let address_unlock_condition = UnlockCondition::Address(AddressUnlockCondition::new(*address));
    // Safety: This can never fail because the amount will always be within the valid range. Also, the actual value is
    // not important, we are only interested in the storage requirements of the type.
    let mut nft_builder = NftOutputBuilder::new_with_amount(OutputAmount::MIN, NftId::null())?
        .add_unlock_condition(address_unlock_condition);
    if let Some(immutable_metadata) = immutable_metadata {
        nft_builder = nft_builder.add_immutable_feature_block(immutable_metadata);
    }
    if let Some(metadata) = metadata {
        nft_builder = nft_builder.add_feature_block(metadata);
    }
    Ok(Output::Nft(nft_builder.finish()?).byte_cost(config))
}
