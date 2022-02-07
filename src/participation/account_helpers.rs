// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::address::{Address, AddressOutput};
use iota_client::node_manager::Node;

use std::collections::HashMap;

// helper function to get the staked funds and participations
pub(crate) async fn get_outputs_participation(
    address_outputs: Vec<AddressOutput>,
    node: Node,
    assembly_event_id: &str,
) -> crate::Result<(
    u64,
    u64,
    HashMap<String, crate::participation::response_types::TrackedParticipation>,
)> {
    let mut total_output_participation: HashMap<String, crate::participation::response_types::TrackedParticipation> =
        HashMap::new();
    let shimmer_staked_funds = 0;
    let mut assembly_staked_funds = 0;

    // We split the outputs into chunks so we don't get timeouts if we have thousands
    for output_ids_chunk in address_outputs.chunks(100).map(|x: &[AddressOutput]| x.to_vec()) {
        let mut tasks = Vec::new();
        for output in output_ids_chunk {
            let node = node.clone();
            let output_id = output.id()?.to_string();
            tasks.push(async move {
                tokio::spawn(async move {
                    (
                        output,
                        crate::participation::endpoints::get_output_participation(node.clone(), output_id).await,
                    )
                })
                .await
            });
        }
        let results = futures::future::try_join_all(tasks).await?;
        for res in results {
            let (output, output_participation_res) = res;
            if let Ok(output_participation) = output_participation_res {
                for (event_id, _participation) in output_participation.participations {
                    if event_id == assembly_event_id {
                        assembly_staked_funds += output.amount;
                    }
                    total_output_participation
                        .entry(event_id)
                        .and_modify(|p| p.amount += participation.amount)
                        .or_insert_with(|| participation);
                }
            }
        }
    }

    Ok((shimmer_staked_funds, assembly_staked_funds, total_output_participation))
}

// helper function to get the rewards
pub(crate) async fn get_addresses_staking_rewards(
    addresses: Vec<Address>,
    node: Node,
    assembly_event_id: &str,
) -> crate::Result<(u64, u64, u64, u64)> {
    let mut assembly_rewards = 0;
    let mut assembly_rewards_below_minimum = 0;
    // We split the addresses into chunks so we don't get timeouts if we have thousands
    for addresses_chunk in addresses.chunks(100).map(|x: &[Address]| x.to_vec()) {
        let mut tasks = Vec::new();
        for address in addresses_chunk {
            let node = node.clone();
            tasks.push(async move {
                tokio::spawn(async move {
                    let staking_status = crate::participation::endpoints::get_address_staking_status(
                        node.clone(),
                        address.address().to_bech32(),
                    )
                    .await?;
                    crate::Result::Ok(staking_status)
                })
                .await
            });
        }
        let results = futures::future::try_join_all(tasks).await?;
        for res in results {
            let staking_status = res?;
            for (event_id, status) in staking_status.rewards {
                if event_id == assembly_event_id {
                    if status.minimum_reached {
                        assembly_rewards += status.amount
                    } else {
                        assembly_rewards_below_minimum += status.amount
                    }
                }
            }
        }
    }

    Ok((0, assembly_rewards, 0, assembly_rewards_below_minimum))
}
