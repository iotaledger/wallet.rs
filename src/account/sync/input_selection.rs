use rand::{thread_rng, Rng};

pub fn select_input(target: u64, available_utxos: &mut Vec<u64>) -> crate::Result<Vec<u64>> {
    if target > available_utxos.iter().sum::<u64>() {
        return Err(anyhow::anyhow!("insufficient funds"));
    }

    available_utxos.sort_by(|a, b| b.cmp(&a));
    let mut selected_coins = Vec::new();
    let result = branch_and_bound(target, available_utxos, 0, &mut selected_coins, 0, 100000);

    if result {
        Ok(selected_coins)
    } else {
        // If no match, Single Random Draw
        single_random_draw(target, available_utxos)
    }
}

fn single_random_draw(target: u64, available_utxos: &mut Vec<u64>) -> crate::Result<Vec<u64>> {
    thread_rng().shuffle(available_utxos);
    let mut sum = 0;

    let selected_coins = available_utxos
        .iter_mut()
        .map(|x| *x)
        .take_while(|x| {
            let value = *x;
            let old_sum = sum;
            sum += value;
            old_sum < target
        })
        .collect();

    Ok(selected_coins)
}

fn branch_and_bound(
    target: u64,
    available_utxos: &mut Vec<u64>,
    depth: usize,
    current_selection: &mut Vec<u64>,
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
    let current_utxo_value = available_utxos[depth];
    current_selection.push(available_utxos[depth]);

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
    use rand::{Rng, SeedableRng, StdRng};

    fn generate_random_utxos(rng: &mut StdRng, utxos_number: i32) -> Vec<u64> {
        let mut available_utxos = Vec::new();
        for _i in 0..utxos_number {
            available_utxos.push(rng.gen_range(0, 2000));
        }
        available_utxos
    }

    fn sum_random_utxos<'a>(rng: &mut StdRng, available_utxos: &mut Vec<u64>) -> u64 {
        let utxos_picked_len = rng.gen_range(2, available_utxos.len() / 2);
        thread_rng().shuffle(available_utxos);
        available_utxos[..utxos_picked_len]
            .iter()
            .fold(0, |acc, x| acc + x)
    }

    #[test]
    fn exact_match() {
        let seed: &[_] = &[1, 2, 3, 4];
        let mut rng: StdRng = SeedableRng::from_seed(seed);
        for _i in 0..20 {
            let mut available_utxos = generate_random_utxos(&mut rng, 30);
            let sum_utxos_picked = sum_random_utxos(&mut rng, &mut available_utxos);
            let selected = select_input(sum_utxos_picked, &mut available_utxos).unwrap();
            assert_eq!(
                selected.into_iter().fold(0, |acc, x| acc + x),
                sum_utxos_picked
            );
        }
    }

    #[test]
    fn insufficient_funds() {
        let seed: &[_] = &[1, 2, 3, 4];
        let mut rng: StdRng = SeedableRng::from_seed(seed);
        let mut available_utxos = generate_random_utxos(&mut rng, 30);
        let target = available_utxos.iter().sum::<u64>() + 1;
        let response = select_input(target, &mut available_utxos);
        assert!(response.is_err());
    }

    #[test]
    fn random_target() {
        let seed: &[_] = &[1, 2, 3, 4];
        let mut rng: StdRng = SeedableRng::from_seed(seed);
        for _ in 0..20 {
            let mut available_utxos = generate_random_utxos(&mut rng, 30);
            let sum_utxos = available_utxos.iter().sum::<u64>();
            let target = rng.gen_range(sum_utxos / 2, sum_utxos * 2);
            let response = select_input(target, &mut available_utxos);
            if target > sum_utxos {
                assert!(response.is_err());
            } else {
                assert!(response.is_ok());
                let selected = response.unwrap();
                assert!(selected.into_iter().fold(0, |acc, x| acc + x) >= target);
            }
        }
    }
}
