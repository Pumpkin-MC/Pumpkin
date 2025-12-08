use std::net::IpAddr;

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::net::GameProfile;

#[derive(Debug, Serialize, Deserialize)]
pub struct BannedPlayerEntry {
    pub uuid: Uuid,
    pub name: String,
    #[serde(with = "format::date")]
    pub created: time::OffsetDateTime,
    pub source: String,
    #[serde(with = "format::option_date")]
    pub expires: Option<time::OffsetDateTime>,
    pub reason: String,
}

impl BannedPlayerEntry {
    #[must_use]
    pub fn new(
        profile: &GameProfile,
        source: String,
        expires: Option<time::OffsetDateTime>,
        reason: String,
    ) -> Self {
        Self {
            uuid: profile.id,
            name: profile.name.clone(),
            created: OffsetDateTime::now_utc(),
            source,
            expires,
            reason,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BannedIpEntry {
    pub ip: IpAddr,
    #[serde(with = "format::date")]
    pub created: time::OffsetDateTime,
    pub source: String,
    #[serde(with = "format::option_date")]
    pub expires: Option<time::OffsetDateTime>,
    pub reason: String,
}

impl BannedIpEntry {
    #[must_use]
    pub fn new(
        ip: IpAddr,
        source: String,
        expires: Option<time::OffsetDateTime>,
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
            let s = date.format(DATE_FORMAT).unwrap();
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

        use crate::data::banlist_serializer::format::DATE_FORMAT;

        #[allow(clippy::ref_option)]
        pub fn serialize<S: Serializer>(
            date: &Option<OffsetDateTime>,
            serializer: S,
        ) -> Result<S::Ok, S::Error> {
            if let Some(date) = date {
                let s = date.format(DATE_FORMAT).unwrap();
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
