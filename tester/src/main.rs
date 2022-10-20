// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod error;

use serde_json::Value;
use tokio::fs;

use self::error::Error;

fn process_fixtures(fixtures: &Value) -> Result<(), Error> {
    println!("{}", fixtures);

    Ok(())
}

fn process_transactions(transactions: &Value) -> Result<(), Error> {
    println!("{}", transactions);

    Ok(())
}

fn process_tests(tests: &Value) -> Result<(), Error> {
    println!("{}", tests);

    Ok(())
}

fn process_json(json: Value) -> Result<(), Error> {
    if let Some(fixtures) = json.get("fixtures") {
        process_fixtures(fixtures)?;
    }

    if let Some(transactions) = json.get("transactions") {
        process_transactions(transactions)?;
    }

    if let Some(tests) = json.get("tests") {
        process_tests(tests)?;
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
