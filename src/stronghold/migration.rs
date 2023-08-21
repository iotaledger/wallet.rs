// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{
    ffi::OsStr,
    num::NonZeroU32,
    path::{Path, PathBuf},
};

use zeroize::Zeroize;

use crate::{actor::AccountManager, Result, StrongholdError};

impl AccountManager {
    /// Migrates a stronghold snapshot from version 2 to version 3.
    pub fn migrate_stronghold_snapshot_v2_to_v3<P: AsRef<Path>>(
        current_path: P,
        current_password: &str,
        salt: &str,
        rounds: u32,
        new_path: Option<P>,
        new_password: Option<&str>,
    ) -> Result<()> {
        log::debug!("migrate_v2_to_v3");
        use iota_stronghold::engine::snapshot::migration::{migrate, Version};

        let mut buffer = [0u8; 32];
        let mut tmp_path = current_path.as_ref().as_os_str().to_os_string();
        tmp_path.push(OsStr::new("-tmp"));
        let tmp_path = PathBuf::from(tmp_path);

        if tmp_path.exists() {
            return Err(StrongholdError::PathAlreadyExists(tmp_path))?;
        }

        crypto::keys::pbkdf::PBKDF2_HMAC_SHA512(
            current_password.as_bytes(),
            salt.as_bytes(),
            NonZeroU32::try_from(rounds).map_err(|_| StrongholdError::InvalidRounds(rounds))?,
            buffer.as_mut(),
        );

        let current_version = Version::V2 {
            path: current_path.as_ref(),
            key: &buffer,
            aad: &[],
        };

        let new_password = new_password.unwrap_or(current_password);
        let new_version = Version::V3 {
            path: &tmp_path,
            password: new_password.as_bytes(),
        };

        migrate(current_version, new_version).map_err(crate::stronghold::Error::Migration)?;

        let new_path = new_path.unwrap_or(current_path);
        std::fs::rename(tmp_path, new_path.as_ref())?;

        buffer.zeroize();

        Ok(())
    }
}
