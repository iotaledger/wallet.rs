// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::address::AddressWrapper;
use rand::{prelude::SliceRandom, thread_rng};
use std::{cmp::Ordering, convert::TryInto};

#[derive(Debug, Clone)]
pub struct Input {
    pub address: AddressWrapper,
    pub internal: bool,
    pub balance: u64,
}

pub fn select_input(target: u64, mut available_utxos: Vec<Input>) -> crate::Result<Vec<Input>> {
    if target > available_utxos.iter().fold(0, |acc, address| acc + address.balance) {
        return Err(crate::Error::InsufficientFunds);
    }

    available_utxos.sort_by(|a, b| match b.balance.cmp(&a.balance) {
        // if the balances are equal, we prioritise change addresses
        Ordering::Equal => b.internal.cmp(&a.internal),
        Ordering::Greater => Ordering::Greater,
        Ordering::Less => Ordering::Less,
    });
    let mut selected_coins = Vec::new();
    let tries = 2i64
        .checked_pow(available_utxos.len().try_into().unwrap())
        .unwrap_or(i64::max_value());
    let result = branch_and_bound(target, &mut available_utxos, 0, &mut selected_coins, 0, tries);

    if result {
        Ok(selected_coins)
    } else {
        // If no match, Single Random Draw
        Ok(single_random_draw(target, available_utxos))
    }
}

fn single_random_draw(target: u64, mut available_utxos: Vec<Input>) -> Vec<Input> {
    available_utxos.shuffle(&mut thread_rng());
    let mut sum = 0;

    available_utxos
        .into_iter()
        .take_while(|address| {
            let value = address.balance;
            let old_sum = sum;
            sum += value;
            old_sum < target
        })
        .collect()
}

fn branch_and_bound(
    target: u64,
    available_utxos: &mut Vec<Input>,
    depth: usize,
    current_selection: &mut Vec<Input>,
    effective_value: u64,
    mut tries: i64,
) -> bool {
    if effective_value > target {
        return false;
    }

    if effective_value == target {
        return true;
    }

    if tries <= 0 || depth >= available_utxos.len() {
        return false;
    }

    tries -= 1;

    // Exploring omission and inclusion branch
    let current_utxo_value = available_utxos[depth].balance;
    current_selection.push(available_utxos[depth].clone());

    if branch_and_bound(
        target,
        available_utxos,
        depth + 1,
        current_selection,
        effective_value + current_utxo_value,
        tries,
    ) {
        return true;
    }

    current_selection.pop();

    branch_and_bound(
        target,
        available_utxos,
        depth + 1,
        current_selection,
        effective_value,
        tries,
    )
}

// TODO use quickcheck or proptest
#[cfg(test)]
mod tests {
    use super::*;
    use crate::address::{AddressBuilder, AddressWrapper, IotaAddress};
    use iota::message::prelude::Ed25519Address;
    use rand::prelude::{Rng, SeedableRng, SliceRandom, StdRng};

    fn generate_random_utxos(rng: &mut StdRng, utxos_number: usize) -> Vec<Input> {
        let mut available_utxos = Vec::new();
        for i in 0..utxos_number {
            let address = AddressBuilder::new()
                .address(AddressWrapper::new(
                    IotaAddress::Ed25519(Ed25519Address::new([0; 32])),
                    "iota".to_string(),
                ))
                .balance(rng.gen_range(0..2000))
                .key_index(i)
                .outputs(vec![])
                .build()
                .unwrap();
            available_utxos.push(super::Input {
                address: address.address().clone(),
                balance: *address.balance(),
                internal: false,
            });
        }
        available_utxos
    }

    fn sum_random_utxos(rng: &mut StdRng, available_utxos: &mut Vec<Input>) -> u64 {
        let utxos_picked_len = rng.gen_range(2..available_utxos.len() / 2);
        available_utxos.shuffle(&mut thread_rng());
        available_utxos[..utxos_picked_len]
            .iter()
            .fold(0, |acc, address| acc + address.balance)
    }

    #[test]
    fn exact_match() {
        let seed: [u8; 32] = [1; 32];
        let mut rng: StdRng = SeedableRng::from_seed(seed);
        for _i in 0..20 {
            let mut available_utxos = generate_random_utxos(&mut rng, 30);
            let sum_utxos_picked = sum_random_utxos(&mut rng, &mut available_utxos);
            let selected = select_input(sum_utxos_picked, available_utxos).unwrap();
            assert_eq!(
                selected.iter().fold(0, |acc, address| { acc + address.balance }),
                sum_utxos_picked
            );
        }
    }

    #[test]
    fn non_exact_match() {
        let seed: [u8; 32] = [1; 32];
        let mut rng: StdRng = SeedableRng::from_seed(seed);
        for _i in 0..20 {
            let available_utxos = generate_random_utxos(&mut rng, 5);
            let target = available_utxos.iter().fold(0, |acc, address| acc + address.balance) - 1;
            let selected = select_input(target, available_utxos).unwrap();
            assert!(selected.into_iter().fold(0, |acc, address| acc + address.balance) >= target);
        }
    }

    #[test]
    fn insufficient_funds() {
        let seed: [u8; 32] = [1; 32];
        let mut rng: StdRng = SeedableRng::from_seed(seed);
        let available_utxos = generate_random_utxos(&mut rng, 30);
        let target = available_utxos.iter().fold(0, |acc, address| acc + address.balance) + 1;
        let response = select_input(target, available_utxos);
        assert!(response.is_err());
    }

    #[test]
    fn random_target() {
        let seed: [u8; 32] = [1; 32];
        let mut rng: StdRng = SeedableRng::from_seed(seed);
        for _ in 0..20 {
            let available_utxos = generate_random_utxos(&mut rng, 30);
            let sum_utxos = available_utxos.iter().fold(0, |acc, address| acc + address.balance);
            let target = rng.gen_range(sum_utxos / 2..sum_utxos * 2);
            let response = select_input(target, available_utxos);
            if target > sum_utxos {
                assert!(response.is_err());
            } else {
                assert!(response.is_ok());
                let selected = response.unwrap();
                assert!(selected.into_iter().fold(0, |acc, address| acc + address.balance) >= target);
            }
        }
    }
}
