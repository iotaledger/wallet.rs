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

pub(crate) mod message_id_serde {
    use iota::message::prelude::MessageId;
    use serde::{
        de::{Error as DeError, Visitor},
        Deserializer, Serializer,
    };
    use std::convert::TryInto;

    pub fn serialize<S: Serializer>(id: &MessageId, s: S) -> std::result::Result<S::Ok, S::Error> {
        s.serialize_str(&id.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<MessageId, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MessageIdVisitor;
        impl<'de> Visitor<'de> for MessageIdVisitor {
            type Value = MessageId;
            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a message id as hex string")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let decoded = hex::decode(v).map_err(|e| DeError::custom(e.to_string()))?;
                let id = MessageId::new(
                    decoded[..]
                        .try_into()
                        .map_err(|_| DeError::custom("invalid serialized message id length"))?,
                );
                Ok(id)
            }
        }
        deserializer.deserialize_str(MessageIdVisitor)
    }
}

impl serde::Serialize for crate::WalletError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        fn serialize_variant<S: Serializer>(
            serializer: S,
            variant_name: &str,
            message: Option<&str>,
        ) -> std::result::Result<S::Ok, S::Error> {
            let mut state = serializer.serialize_struct("WalletError", 2)?;
            state.serialize_field("type", variant_name)?;
            state.serialize_field("error", &message)?;
            state.end()
        }
        match self {
            Self::UnknownError(error) => serialize_variant(serializer, "UnknownError", Some(error)),
            Self::GenericError(error) => serialize_variant(serializer, "GenericError", Some(&error.to_string())),
            Self::IoError(error) => serialize_variant(serializer, "IoError", Some(&error.to_string())),
            Self::JsonError(error) => serialize_variant(serializer, "JsonError", Some(&error.to_string())),
            #[cfg(feature = "stronghold")]
            Self::StrongholdError(error) => serialize_variant(serializer, "StrongholdError", Some(&error.to_string())),
            Self::ClientError(error) => serialize_variant(serializer, "ClientError", Some(&error.to_string())),
            #[cfg(feature = "sqlite")]
            Self::SqliteError(error) => serialize_variant(serializer, "SqliteError", Some(&error.to_string())),
            Self::UrlError(error) => serialize_variant(serializer, "UrlError", Some(&error.to_string())),
            Self::UnexpectedResponse(error) => serialize_variant(serializer, "UnexpectedResponse", Some(&error)),
            Self::MessageAboveMaxDepth => serialize_variant(serializer, "MessageAboveMaxDepth", None),
            Self::MessageAlreadyConfirmed => serialize_variant(serializer, "MessageAlreadyConfirmed", None),
            Self::MessageNotFound => serialize_variant(serializer, "MessageNotFound", None),
            Self::EmptyNodeList => serialize_variant(serializer, "EmptyNodeList", None),
            Self::InvalidAddressLength => serialize_variant(serializer, "InvalidAddressLength", None),
            Self::InvalidTransactionIdLength => {
                serializer.serialize_newtype_variant("WalletError", 14, "InvalidTransactionIdLength", "")
            }
            Self::InvalidMessageIdLength => serialize_variant(serializer, "InvalidMessageIdLength", None),
            Self::Bech32Error(error) => serialize_variant(serializer, "Bech32Error", Some(&error.to_string())),
            Self::AccountAlreadyImported { alias } => serialize_variant(
                serializer,
                "AccountAlreadyImported",
                Some(&format!("account {} already imported", alias)),
            ),
            Self::StorageDoesntExist => serialize_variant(serializer, "StorageDoesntExist", None),
            Self::InsufficientFunds => serialize_variant(serializer, "InsufficientFunds", None),
            Self::MessageNotEmpty => serialize_variant(serializer, "MessageNotEmpty", None),
            Self::LatestAccountIsEmpty => serialize_variant(serializer, "LatestAccountIsEmpty", None),
            Self::ZeroAmount => serialize_variant(serializer, "ZeroAmount", None),
            Self::AccountNotFound => serialize_variant(serializer, "AccountNotFound", None),
            Self::InvalidRemainderValueAddress => serialize_variant(serializer, "InvalidRemainderValueAddress", None),
            Self::StrongholdNotInitialised => serialize_variant(serializer, "StrongholdNotInitialised", None),
        }
    }
}
