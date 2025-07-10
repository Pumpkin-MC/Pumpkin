use serde::{Serialize, Serializer};

pub struct Le64(pub i64);

impl Serialize for Le64 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(&self.0.to_le_bytes())
    }
}

pub struct Le32(pub i32);

impl Serialize for Le32 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(&self.0.to_le_bytes())
    }
}

pub struct Le16(pub i16);

impl Serialize for Le16 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(&self.0.to_le_bytes())
    }
}

// Unsigned

pub struct LeU64(pub u64);

impl Serialize for LeU64 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(&self.0.to_le_bytes())
    }
}

pub struct LeU32(pub u32);

impl Serialize for LeU32 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(&self.0.to_le_bytes())
    }
}

pub struct LeU16(pub u16);

impl Serialize for LeU16 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(&self.0.to_le_bytes())
    }
}
