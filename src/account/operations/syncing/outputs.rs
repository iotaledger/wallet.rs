// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::account::{handle::AccountHandle, types::OutputData, AddressWithBalance};

use crypto::keys::slip10::Chain;
use iota_client::{
    api::ClientMessageBuilder,
    bee_message::{output::OutputId, payload::transaction::TransactionId},
    bee_rest_api::types::responses::OutputResponse,
};

use std::{str::FromStr, time::Instant};

/// Convert OutputResponse to OutputData with the network_id added
pub(crate) async fn output_response_to_output_data(
    account_handle: &AccountHandle,
    output_responses: Vec<OutputResponse>,
    associated_address: &AddressWithBalance,
) -> crate::Result<Vec<OutputData>> {
    log::debug!("[SYNC] convert output_responses");
    // store outputs with network_id
    let account = account_handle.read().await;
    let network_id = account_handle.client.get_network_id().await?;
    let bech32_hrp = account_handle.client.get_bech32_hrp().await?;
    let mut outputs = Vec::new();
    for output in output_responses {
        let (amount, address) = ClientMessageBuilder::get_output_amount_and_address(&output.output, None)?;
        let transaction_id = TransactionId::from_str(&output.transaction_id)?;
        // check if we know the transaction that created this output and if we created it (if we store incoming
        // transactions separated, then this check wouldn't be required)
        let remainder = {
            match account.transactions.get(&transaction_id) {
                Some(tx) => !tx.incoming,
                None => false,
            }
        };

        // 44 is for BIP 44 (HD wallets) and 4218 is the registered index for IOTA https://github.com/satoshilabs/slips/blob/master/slip-0044.md
        let chain = Chain::from_u32_hardened(vec![
            44,
            account.coin_type,
            account.index,
            associated_address.internal as u32,
            associated_address.key_index,
        ]);

        outputs.push(OutputData {
            output_id: OutputId::new(transaction_id, output.output_index)?,
            output_response: output.clone(),
            amount,
            is_spent: output.is_spent,
            address,
            network_id,
            remainder,
            chain: Some(chain),
        });
    }
    Ok(outputs)
}

/// Get the current output ids for provided addresses
pub(crate) async fn get_outputs(
    account_handle: &AccountHandle,
    output_ids: Vec<OutputId>,
) -> crate::Result<Vec<OutputResponse>> {
    log::debug!("[SYNC] start get_outputs");
    let get_outputs_sync_start_time = Instant::now();
    let account = account_handle.read().await;

    drop(account);

    let found_outputs = account_handle.client.get_outputs(output_ids).await?;

    log::debug!(
        "[SYNC] finished get_outputs in {:.2?}",
        get_outputs_sync_start_time.elapsed()
    );
    Ok(found_outputs)
}
