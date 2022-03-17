// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::Result;

use iota_client::bee_message::{
    address::{Address, AliasAddress},
    milestone::MilestoneIndex,
    output::{
        unlock_condition::{
            AddressUnlockCondition, ExpirationUnlockCondition, GovernorAddressUnlockCondition,
            ImmutableAliasAddressUnlockCondition, StateControllerAddressUnlockCondition,
            StorageDepositReturnUnlockCondition, UnlockCondition,
        },
        AliasId, AliasOutputBuilder, BasicOutputBuilder, ByteCost, ByteCostConfig, FoundryOutputBuilder, NativeToken,
        Output, TokenId, TokenScheme, TokenTag,
    },
};
use primitive_types::U256;

// todo: move to bee-message/iota.rs

/// Computes the minimum amount that a storage deposit has to match to allow creating a return [`Output`] back to the
/// sender [`Address`].
pub(crate) fn minimum_storage_deposit_basic(config: &ByteCostConfig, address: &Address) -> u64 {
    let address_condition = UnlockCondition::Address(AddressUnlockCondition::new(*address));
    // Safety: This can never fail because the amount will always be within the valid range. Also, the actual value is
    // not important, we are only interested in the storage requirements of the type.
    // todo: use `OutputAmount::MIN` when public, see https://github.com/iotaledger/bee/issues/1238
    let basic_output = BasicOutputBuilder::new(1_000_000_000)
        .unwrap()
        .add_unlock_condition(address_condition)
        .finish()
        .unwrap();
    Output::Basic(basic_output).byte_cost(config)
}

/// Computes the minimum amount that an alias output needs to have.
pub(crate) fn minimum_storage_deposit_alias(config: &ByteCostConfig, address: &Address) -> Result<u64> {
    let address_condition = UnlockCondition::Address(AddressUnlockCondition::new(*address));
    // Safety: This can never fail because the amount will always be within the valid range. Also, the actual value is
    // not important, we are only interested in the storage requirements of the type.
    // todo: use `OutputAmount::MIN` when public, see https://github.com/iotaledger/bee/issues/1238
    let alias_output = AliasOutputBuilder::new(1_000_000_000, AliasId::from([0; 20]))?
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
    // todo: use `OutputAmount::MIN` when public, see https://github.com/iotaledger/bee/issues/1238
    let foundry_output = FoundryOutputBuilder::new(
        1_000_000_000,
        1,
        TokenTag::new([0u8; 12]),
        U256::from(0),
        U256::from(1),
        TokenScheme::Simple,
    )?
    .add_unlock_condition(UnlockCondition::ImmutableAliasAddress(
        ImmutableAliasAddressUnlockCondition::new(AliasAddress::new(AliasId::new([0u8; 20]))),
    ))
    .finish()?;
    Ok(Output::Foundry(foundry_output).byte_cost(config))
}

/// Computes the minimum amount that an output needs to have, when native tokens are sent with [AddressUnlockCondition],
/// [StorageDepositReturnUnlockCondition] and [ExpirationUnlockCondition].
pub(crate) fn minimum_storage_deposit_basic_native_tokens(
    config: &ByteCostConfig,
    address: &Address,
    return_address: &Address,
    native_tokens: &[(TokenId, U256)],
) -> Result<u64> {
    let address_condition = UnlockCondition::Address(AddressUnlockCondition::new(*address));
    // Safety: This can never fail because the amount will always be within the valid range. Also, the actual value is
    // not important, we are only interested in the storage requirements of the type.
    // todo: use `OutputAmount::MIN` when public, see https://github.com/iotaledger/bee/issues/1238
    let basic_output = BasicOutputBuilder::new(1_000_000_000)?
        .with_native_tokens(
            native_tokens
                .iter()
                .map(|(id, amount)| {
                    NativeToken::new(*id, *amount).map_err(|e| crate::Error::ClientError(Box::new(e.into())))
                })
                .collect::<Result<Vec<NativeToken>>>()?,
        )
        .add_unlock_condition(address_condition)
        .add_unlock_condition(UnlockCondition::StorageDepositReturn(
            StorageDepositReturnUnlockCondition::new(*return_address, 1_000_000_000)?,
        ))
        .add_unlock_condition(UnlockCondition::Expiration(ExpirationUnlockCondition::new(
            *return_address,
            // Both 0 would be invalid, so we just use 1
            MilestoneIndex::new(1),
            0,
        )?))
        .finish()?;
    Ok(Output::Basic(basic_output).byte_cost(config))
}
