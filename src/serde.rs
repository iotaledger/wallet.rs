// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub(crate) mod iota_address_serde {
    use crate::address::AddressWrapper;
    use serde::{de::Visitor, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(address: &AddressWrapper, s: S) -> std::result::Result<S::Ok, S::Error> {
        s.serialize_str(&address.to_bech32())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<AddressWrapper, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct AddressVisitor;
        impl<'de> Visitor<'de> for AddressVisitor {
            type Value = AddressWrapper;
            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a bech32 formatted string")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                crate::address::parse(v).map_err(|e| serde::de::Error::custom(e.to_string()))
            }
        }

        deserializer.deserialize_str(AddressVisitor)
    }
}
