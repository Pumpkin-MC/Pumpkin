//! Entry types for `banned-players.json` and `banned-ips.json`.
//!
//! The on-disk format uses a peculiar timestamp layout (not RFC3339) and an
//! `"expires": "forever"` sentinel for permanent bans — both handled by the
//! `format` submodule below.

use std::net::IpAddr;

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BannedPlayerEntry {
    pub uuid: Uuid,
    pub name: String,
    #[serde(with = "format::date")]
    pub created: OffsetDateTime,
    pub source: String,
    #[serde(with = "format::option_date")]
    pub expires: Option<OffsetDateTime>,
    pub reason: String,
}

impl BannedPlayerEntry {
    #[must_use]
    pub fn new(
        uuid: Uuid,
        name: String,
        source: String,
        expires: Option<OffsetDateTime>,
        reason: String,
    ) -> Self {
        Self {
            uuid,
            name,
            created: OffsetDateTime::now_utc(),
            source,
            expires,
            reason,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BannedIpEntry {
    pub ip: IpAddr,
    #[serde(with = "format::date")]
    pub created: OffsetDateTime,
    pub source: String,
    #[serde(with = "format::option_date")]
    pub expires: Option<OffsetDateTime>,
    pub reason: String,
}

impl BannedIpEntry {
    #[must_use]
    pub fn new(
        ip: IpAddr,
        source: String,
        expires: Option<OffsetDateTime>,
        reason: String,
    ) -> Self {
        Self {
            ip,
            created: OffsetDateTime::now_utc(),
            source,
            expires,
            reason,
        }
    }
}

mod format {
    const DATE_FORMAT: &[time::format_description::FormatItem<'static>] = time::macros::format_description!(
        "[year]-[month]-[day] [hour]:[minute]:[second][offset_hour sign:mandatory]:[offset_minute]"
    );

    pub mod date {
        use serde::{self, Deserialize, Deserializer, Serializer};
        use time::OffsetDateTime;

        use super::DATE_FORMAT;

        pub fn serialize<S: Serializer>(
            date: &OffsetDateTime,
            serializer: S,
        ) -> Result<S::Ok, S::Error> {
            let s = date
                .format(DATE_FORMAT)
                .expect("const format descriptor is always valid");
            serializer.serialize_str(&s)
        }

        pub fn deserialize<'de, D: Deserializer<'de>>(
            deserializer: D,
        ) -> Result<OffsetDateTime, D::Error> {
            let s = String::deserialize(deserializer)?;
            OffsetDateTime::parse(&s, DATE_FORMAT).map_err(serde::de::Error::custom)
        }
    }

    pub mod option_date {
        use serde::{self, Deserialize, Deserializer, Serializer};
        use time::OffsetDateTime;

        use super::DATE_FORMAT;

        #[expect(clippy::ref_option)]
        pub fn serialize<S: Serializer>(
            date: &Option<OffsetDateTime>,
            serializer: S,
        ) -> Result<S::Ok, S::Error> {
            if let Some(date) = date {
                let s = date
                    .format(DATE_FORMAT)
                    .expect("const format descriptor is always valid");
                serializer.serialize_str(&s)
            } else {
                serializer.serialize_str("forever")
            }
        }

        pub fn deserialize<'de, D: Deserializer<'de>>(
            deserializer: D,
        ) -> Result<Option<OffsetDateTime>, D::Error> {
            let s = String::deserialize(deserializer)?;
            if s == "forever" {
                Ok(None)
            } else {
                OffsetDateTime::parse(&s, DATE_FORMAT)
                    .map(Some)
                    .map_err(serde::de::Error::custom)
            }
        }
    }
}
