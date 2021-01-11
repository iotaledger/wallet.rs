// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use serde::{ser::SerializeStruct, Serializer};

pub(crate) mod iota_address_serde {
    use crate::address::IotaAddress;
    use bech32::FromBase32;
    use iota::message::prelude::Ed25519Address;
    use serde::{de::Visitor, Deserializer, Serializer};
    use std::convert::TryInto;

    pub fn serialize<S: Serializer>(address: &IotaAddress, s: S) -> std::result::Result<S::Ok, S::Error> {
        s.serialize_str(&address.to_bech32())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<IotaAddress, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct AddressVisitor;
        impl<'de> Visitor<'de> for AddressVisitor {
            type Value = IotaAddress;
            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a bech32 formatted string")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let decoded = Vec::from_base32(
                    &bech32::decode(v)
                        .map_err(|e| serde::de::Error::custom(e.to_string()))?
                        .1,
                )
                .map_err(|e| serde::de::Error::custom(e.to_string()))?;
                let address_type = decoded[0];
                match address_type {
                    1 => Ok(IotaAddress::Ed25519(Ed25519Address::new(
                        decoded[1..]
                            .try_into()
                            .map_err(|_| serde::de::Error::custom("invalid address length"))?,
                    ))),
                    _ => Err(serde::de::Error::custom("invalid address type")),
                }
            }
        }

        deserializer.deserialize_str(AddressVisitor)
    }
}

impl serde::Serialize for crate::Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        fn serialize_variant<S: Serializer>(
            error: &crate::Error,
            serializer: S,
            variant_name: &str,
        ) -> std::result::Result<S::Ok, S::Error> {
            let mut state = serializer.serialize_struct("Error", 2)?;
            state.serialize_field("type", variant_name)?;
            state.serialize_field("error", &error.to_string())?;
            state.end()
        }

        match self {
            Self::IoError(_) => serialize_variant(self, serializer, "IoError"),
            Self::JsonError(_) => serialize_variant(self, serializer, "JsonError"),
            #[cfg(any(feature = "stronghold", feature = "stronghold-storage"))]
            Self::StrongholdError(_) => serialize_variant(self, serializer, "StrongholdError"),
            Self::ClientError(error) => serialize_variant(self, serializer, "ClientError"),
            #[cfg(feature = "sqlite-storage")]
            Self::SqliteError(_) => serialize_variant(self, serializer, "SqliteError"),
            Self::UrlError(_) => serialize_variant(self, serializer, "UrlError"),
            Self::UnexpectedResponse(_) => serialize_variant(self, serializer, "UnexpectedResponse"),
            Self::MessageAboveMaxDepth => serialize_variant(self, serializer, "MessageAboveMaxDepth"),
            Self::MessageAlreadyConfirmed => serialize_variant(self, serializer, "MessageAlreadyConfirmed"),
            Self::MessageNotFound => serialize_variant(self, serializer, "MessageNotFound"),
            Self::EmptyNodeList => serialize_variant(self, serializer, "EmptyNodeList"),
            Self::InvalidAddressLength => serialize_variant(self, serializer, "InvalidAddressLength"),
            Self::InvalidMessageIdLength => serialize_variant(self, serializer, "InvalidMessageIdLength"),
            Self::Bech32Error(_) => serialize_variant(self, serializer, "Bech32Error"),
            Self::AccountAlreadyImported { alias: _ } => serialize_variant(self, serializer, "AccountAlreadyImported"),
            Self::StorageDoesntExist => serialize_variant(self, serializer, "StorageDoesntExist"),
            Self::InsufficientFunds => serialize_variant(self, serializer, "InsufficientFunds"),
            Self::MessageNotEmpty => serialize_variant(self, serializer, "MessageNotEmpty"),
            Self::LatestAccountIsEmpty => serialize_variant(self, serializer, "LatestAccountIsEmpty"),
            Self::ZeroAmount => serialize_variant(self, serializer, "ZeroAmount"),
            Self::AccountNotFound => serialize_variant(self, serializer, "AccountNotFound"),
            Self::InvalidRemainderValueAddress => serialize_variant(self, serializer, "InvalidRemainderValueAddress"),
            Self::Storage(_) => serialize_variant(self, serializer, "Storage"),
            Self::Panic(_) => serialize_variant(self, serializer, "Panic"),
            Self::TransferDestinationEmpty => serialize_variant(self, serializer, "TransferDestinationEmpty"),
            Self::InvalidMessageId => serialize_variant(self, serializer, "InvalidMessageId"),
            Self::InvalidTransactionId => serialize_variant(self, serializer, "InvalidTransactionId"),
            Self::AddressBuildRequiredField(_) => serialize_variant(self, serializer, "AddressBuildRequiredField"),
            Self::AccountInitialiseRequiredField(_) => {
                serialize_variant(self, serializer, "AccountInitialiseRequiredField")
            }
            #[cfg(any(feature = "stronghold", feature = "stronghold-storage"))]
            Self::StrongholdNotLoaded => serialize_variant(self, serializer, "StrongholdNotLoaded"),
            Self::Hex(_) => serialize_variant(self, serializer, "Hex"),
            Self::BeeMessage(_) => serialize_variant(self, serializer, "BeeMessage"),
            Self::InvalidDerivationPath(_) => serialize_variant(self, serializer, "InvalidDerivationPath"),
            Self::FailedToGeneratePrivateKey(_) => serialize_variant(self, serializer, "FailedToGeneratePrivateKey"),
            Self::ParseDate(_) => serialize_variant(self, serializer, "ParseDate"),
            Self::MnemonicEncode => serialize_variant(self, serializer, "MnemonicEncode"),
            Self::InvalidMnemonic(_) => serialize_variant(self, serializer, "InvalidMnemonic"),
            Self::Crypto(_) => serialize_variant(self, serializer, "Crypto"),
            Self::BackupNotFile => serialize_variant(self, serializer, "BackupNotFile"),
            Self::InvalidBackupDestination => serialize_variant(self, serializer, "InvalidBackupDestination"),
            Self::StorageAdapterNotDefined => serialize_variant(self, serializer, "StorageAdapterNotDefined"),
            Self::StorageExists => serialize_variant(self, serializer, "StorageExists"),
            Self::StorageAdapterNotSet(_) => serialize_variant(self, serializer, "StorageAdapterNotSet"),
        }
    }
}
