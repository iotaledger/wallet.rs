// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::collections::VecDeque;

use dialoguer::History;

pub struct AccountHistory {
    max: usize,
    history: VecDeque<String>,
}

impl Default for AccountHistory {
    fn default() -> Self {
        AccountHistory {
            max: 25,
            history: VecDeque::new(),
        }
    }
}

impl<T: ToString> History<T> for AccountHistory {
    fn read(&self, pos: usize) -> Option<String> {
        self.history.get(pos).cloned()
    }

    fn write(&mut self, val: &T) {
        if self.history.contains(&val.to_string()) {
            return;
        }
        if self.history.len() == self.max {
            self.history.pop_back();
        }
        self.history.push_front(val.to_string());
    }
}
