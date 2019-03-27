// Newtype wrappers that implement ser/de

use crate::common::*;
use core::str::FromStr;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::borrow::Cow;
use url::Url;

#[derive(PartialEq, Clone, Debug)]
pub struct UrlSerDe(pub Url);

impl Serialize for UrlSerDe {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.0.as_str())
    }
}

impl<'de> Deserialize<'de> for UrlSerDe {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let url = <Cow<str>>::deserialize(deserializer)?
            .parse()
            .map_err(de::Error::custom)?;
        Ok(UrlSerDe(url))
    }
}

impl FromStr for UrlSerDe {
    type Err = <Url as FromStr>::Err;
    fn from_str(other: &str) -> Result<Self, <Self as FromStr>::Err> {
        let url: Url = <Url as FromStr>::from_str(other)?;
        Ok(UrlSerDe(url))
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct InvoiceSerDe(pub Invoice);

impl Serialize for InvoiceSerDe {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&to_bolt11(&self.0))
    }
}

impl<'de> Deserialize<'de> for InvoiceSerDe {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let bech32 = <Cow<str>>::deserialize(deserializer)?;
        // ParseOrSemanticError does not implement Display, but it does implement Debug
        let invoice =
            parse_bolt11(&bech32).map_err(|err| de::Error::custom(format!("{:?}", err)))?;
        Ok(InvoiceSerDe(invoice))
    }
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResultSerDe<K, E> {
    Ok(K),
    Error(E),
}

impl<K, E> From<Result<K, E>> for ResultSerDe<K, E> {
    fn from(other: Result<K, E>) -> Self {
        match other {
            Ok(k) => ResultSerDe::Ok(k),
            Err(k) => ResultSerDe::Error(k),
        }
    }
}

impl<K, E> Into<Result<K, E>> for ResultSerDe<K, E> {
    fn into(self) -> Result<K, E> {
        match self {
            ResultSerDe::Ok(k) => Ok(k),
            ResultSerDe::Error(k) => Err(k),
        }
    }
}
