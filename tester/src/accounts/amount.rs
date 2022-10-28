// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_wallet::{account::AccountHandle, AddressWithAmount};

use crate::{context::Context, error::Error};

pub async fn process_amount<'a>(context: &Context<'a>, account: &AccountHandle, amount: u64) -> Result<(), Error> {
    if amount != 0 {
        let transaction = context
            .faucet_account
            .send_amount(
                vec![AddressWithAmount {
                    address: account.addresses().await?[0].address().to_bech32(),
                    amount,
                }],
                None,
            )
            .await?;

        if let Some(block_id) = transaction.block_id {
            context
                .faucet_account
                .retry_until_included(&block_id, Some(1), None)
                .await?;
        }

        context.faucet_account.sync(None).await?;
    }

    Ok(())
}
