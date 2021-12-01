// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};

use std::{hash::Hash, str::FromStr};

/// Each of the node APIs the wallet uses.
#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub enum Api {
    /// `get_tips` API
    GetTips,
    /// `post_message` API
    PostMessage,
    /// `get_output` API
    GetOutput,
    /// `get_balance` API
    GetBalance,
}

impl FromStr for Api {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let t = match s {
            "GetTips" => Self::GetTips,
            "PostMessage" => Self::PostMessage,
            "GetOutput" => Self::GetOutput,
            "GetBalance" => Self::GetBalance,
            _ => return Err(format!("unknown api kind `{}`", s)),
        };
        Ok(t)
    }
}

impl Serialize for Api {
    fn serialize<S: Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        s.serialize_str(match self {
            Self::GetTips => "GetTips",
            Self::PostMessage => "PostMessage",
            Self::GetOutput => "GetOutput",
            Self::GetBalance => "GetBalance",
        })
    }
}

impl<'de> Deserialize<'de> for Api {
    fn deserialize<D>(deserializer: D) -> Result<Api, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct StringVisitor;
        impl<'de> Visitor<'de> for StringVisitor {
            type Value = Api;
            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a string representing an Api")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let value = Api::from_str(v).map_err(serde::de::Error::custom)?;
                Ok(value)
            }
        }
        deserializer.deserialize_str(StringVisitor)
    }
}

impl From<Api> for iota_client::Api {
    fn from(api: Api) -> iota_client::Api {
        match api {
            Api::GetTips => iota_client::Api::GetTips,
            Api::PostMessage => iota_client::Api::PostMessage,
            Api::GetOutput => iota_client::Api::GetOutput,
            Api::GetBalance => iota_client::Api::GetBalance,
        }
    }
}
