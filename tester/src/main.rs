// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod error;

use serde_json::Value;
use tokio::fs;

use self::error::Error;

fn process_json(json: Value) -> Result<(), Error> {
    if let Some(fixtures) = json.get("fixtures") {
        println!("{}", fixtures);
    }

    if let Some(transactions) = json.get("transactions") {
        println!("{}", transactions);
    }

    if let Some(tests) = json.get("tests") {
        println!("{}", tests);
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let mut dir = fs::read_dir("json").await?;

    for entry in dir.next_entry().await? {
        let content = fs::read_to_string(entry.path()).await?;
        let json: Value = serde_json::from_str(&content)?;

        println!("{:?}", entry.file_name());
        println!("{}", json);
        process_json(json)?;
    }

    Ok(())
}
