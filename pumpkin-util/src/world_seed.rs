use crate::{
    math::java_string_hash,
    random::{RandomImpl, get_seed, legacy_rand::LegacyRand},
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Copy)]
pub struct Seed(pub u64);

impl From<&str> for Seed {
    fn from(value: &str) -> Self {
        let trimmed = value.trim();
        let value = if !trimmed.is_empty() {
            let i64_value = trimmed
                .parse::<i64>()
                .unwrap_or_else(|_| java_string_hash(trimmed) as i64);
            Some(i64_value as u64)
        } else {
            None
        };

        Seed(value.unwrap_or_else(|| LegacyRand::from_seed(get_seed()).next_i64() as u64))
    }
}

impl Serialize for Seed {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&(self.0 as i64).to_string())
    }
}

impl<'de> Deserialize<'de> for Seed {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        Ok(Self::from(raw.as_str()))
    }
}
