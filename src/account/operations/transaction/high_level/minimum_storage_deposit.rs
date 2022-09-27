// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_client::block::{
    address::Address,
    output::{
        unlock_condition::{
            AddressUnlockCondition, ExpirationUnlockCondition, StorageDepositReturnUnlockCondition, UnlockCondition,
        },
        BasicOutputBuilder, NativeToken, Output, Rent, RentStructure, TokenId,
    },
};
use primitive_types::U256;

use crate::Result;

// todo: move to bee-block/iota.rs

/// Computes the minimum amount that an output needs to have, when sent with [AddressUnlockCondition],
/// [StorageDepositReturnUnlockCondition] and [ExpirationUnlockCondition].
pub(crate) fn minimum_storage_deposit_basic_native_tokens(
    rent_structure: &RentStructure,
    address: &Address,
    return_address: &Address,
    native_tokens: Option<Vec<(TokenId, U256)>>,
    token_supply: u64,
) -> Result<u64> {
    let address_condition = UnlockCondition::Address(AddressUnlockCondition::new(*address));
    // Safety: This can never fail because the amount will always be within the valid range. Also, the actual value is
    // not important, we are only interested in the storage requirements of the type.
    let mut basic_output_builder = BasicOutputBuilder::new_with_amount(Output::AMOUNT_MIN)?
        .add_unlock_condition(address_condition)
        .add_unlock_condition(UnlockCondition::StorageDepositReturn(
            StorageDepositReturnUnlockCondition::new(*return_address, Output::AMOUNT_MIN, token_supply)?,
        ))
        .add_unlock_condition(UnlockCondition::Expiration(ExpirationUnlockCondition::new(
            *return_address,
            // 0 would be invalid
            1,
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

    Ok(basic_output_builder
        .finish_output(token_supply)?
        .rent_cost(rent_structure))
}
