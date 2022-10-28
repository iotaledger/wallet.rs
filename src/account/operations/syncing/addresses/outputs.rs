// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::time::Instant;

use crate::account::{
    constants::PARALLEL_REQUESTS_AMOUNT, handle::AccountHandle, types::address::AddressWithUnspentOutputs, OutputData,
};

impl AccountHandle {
    /// Get outputs from addresses
    pub(crate) async fn get_outputs_from_address_output_ids(
        &self,
        addresses_with_unspent_outputs: Vec<AddressWithUnspentOutputs>,
    ) -> crate::Result<(Vec<AddressWithUnspentOutputs>, Vec<OutputData>)> {
        log::debug!("[SYNC] start get_outputs_from_address_output_ids");
        let address_outputs_start_time = Instant::now();

        let mut addresses_with_outputs = Vec::new();
        let mut outputs_data = Vec::new();

        // We split the addresses into chunks so we don't get timeouts if we have thousands
        for addresses_chunk in &mut addresses_with_unspent_outputs
            .chunks(PARALLEL_REQUESTS_AMOUNT)
            .map(|x: &[AddressWithUnspentOutputs]| x.to_vec())
        {
            let mut tasks = Vec::new();
            for address in addresses_chunk {
                let account_handle = self.clone();
                tasks.push(async move {
                    tokio::spawn(async move {
                        let output_responses = account_handle.get_outputs(address.output_ids.clone()).await?;

                        let outputs = account_handle
                            .output_response_to_output_data(output_responses, &address)
                            .await?;
                        crate::Result::Ok((address, outputs))
                    })
                    .await
                });
            }
            let results = futures::future::try_join_all(tasks).await?;
            for res in results {
                let (address, outputs): (AddressWithUnspentOutputs, Vec<OutputData>) = res?;
                addresses_with_outputs.push(address);
                outputs_data.extend(outputs.into_iter());
            }
        }
        log::debug!(
            "[SYNC] finished get_outputs_from_address_output_ids in {:.2?}",
            address_outputs_start_time.elapsed()
        );
        Ok((addresses_with_outputs, outputs_data))
    }
}
