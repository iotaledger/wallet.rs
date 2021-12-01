// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_client::common::logger::{logger_init, LoggerConfig, LoggerOutputConfigBuilder};
pub use log::LevelFilter;

/// creates a file in which logs will be written in
pub fn init_logger(filename: &str, levelfilter: LevelFilter) -> crate::Result<()> {
    let output_config = LoggerOutputConfigBuilder::new()
        .name(filename)
        .level_filter(levelfilter);
    let config = LoggerConfig::build().with_output(output_config).finish();
    logger_init(config)?;
    Ok(())
}
