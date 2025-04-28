use num_traits::{FromPrimitive, ToPrimitive};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub fn serialize<S, V>(value: &V, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    V: ToPrimitive,
{
    let value = value
        .to_i8()
        .ok_or_else(|| serde::ser::Error::custom("Invalid enum value"))?;
    value.serialize(serializer)
}

pub fn deserialize<'de, D, V>(deserializer: D) -> Result<V, D::Error>
where
    D: Deserializer<'de>,
    V: FromPrimitive,
{
    let value = Deserialize::deserialize(deserializer)?;
    V::from_i8(value).ok_or_else(|| serde::de::Error::custom("Invalid enum value"))
}
