// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_wallet::Error;

#[test]
fn stringified_error() {
    let error = Error::AccountNotFound("0".into());
    assert_eq!(
        &serde_json::to_string(&error).unwrap(),
        "{\"type\":\"accountNotFound\",\"error\":\"account 0 not found\"}"
    );

    let error = Error::NoOutputsToConsolidate {
        available_outputs: 0,
        consolidation_threshold: 0,
    };
    assert_eq!(
        &serde_json::to_string(&error).unwrap(),
        "{\"type\":\"noOutputsToConsolidate\",\"error\":\"nothing to consolidate: available outputs: 0, consolidation threshold: 0\"}"
    );

    let error = Error::FailedToGetRemainder;
    assert_eq!(
        &serde_json::to_string(&error).unwrap(),
        "{\"type\":\"failedToGetRemainder\",\"error\":\"failed to get remainder address\"}"
    );
}
