// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use tokio::fs;

#[tokio::main]
async fn main() {
    // TODO unwrap
    let mut dir = fs::read_dir("json").await.unwrap();

    // TODO unwrap
    for entry in dir.next_entry().await.unwrap() {
        // TODO unwrap
        let content = fs::read_to_string(entry.path()).await.unwrap();

        println!("{:?}", entry.file_name());
        println!("{}", content);
    }
}
