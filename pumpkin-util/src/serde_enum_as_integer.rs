use num_traits::{FromPrimitive, ToPrimitive};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Serializes a numeric enum value into its corresponding `i8` representation.
///
/// # Arguments
/// * `value` - A reference to the enum value implementing `ToPrimitive`.
/// * `serializer` - The serializer to write the value into.
///
/// # Returns
/// The serialized representation of the enum as an `i8`.
///
/// # Errors
/// Returns an error if the enum cannot be converted into an `i8`.
pub fn serialize<S: Serializer, V: ToPrimitive>(
    value: &V,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let value = value
        .to_i8()
        .ok_or_else(|| serde::ser::Error::custom("Invalid enum value"))?;
    value.serialize(serializer)
}

/// Deserializes a numeric enum value from its `i8` representation.
///
/// # Arguments
/// * `deserializer` - The deserializer to read the value from.
///
/// # Returns
/// The enum value reconstructed from the `i8`.
///
/// # Errors
/// Returns an error if the value cannot be converted into the target enum type.
pub fn deserialize<'de, D: Deserializer<'de>, V: FromPrimitive>(
    deserializer: D,
) -> Result<V, D::Error> {
    let value = Deserialize::deserialize(deserializer)?;
    V::from_i8(value).ok_or_else(|| serde::de::Error::custom("Invalid enum value"))
}
